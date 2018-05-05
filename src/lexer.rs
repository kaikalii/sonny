use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use error::*;

static KEYWORDS: &[&'static str] = &[
    "time",
    "sin",
    "cos",
    "ceil",
    "floor",
    "abs",
    "min",
    "max",
    "log",
    "end",
    "out",
    "dur",
    "w",
    "h",
    "q",
    "e",
    "s",
    "ts",
    "tempo",
    "include",
    "use",
    "window_size",
    "sample_rate",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Operator,
    Id,
    Num,
    NoteString,
    Keyword,
    Delimeter,
    BackLink,
    Dot,
    Rest,
    Done,
    Unknown,
    Empty,
}
impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Operator => write!(f, "operator"),
            Id => write!(f, "id"),
            Num => write!(f, "num"),
            NoteString => write!(f, "pitch"),
            Keyword => write!(f, "keyword"),
            Delimeter => write!(f, "delimeter"),
            BackLink => write!(f, "'!'"),
            Dot => write!(f, "'.'"),
            Rest => write!(f, "'_'"),
            Done => write!(f, "enf of file"),
            Unknown => write!(f, "unknown"),
            Empty => write!(f, "empty"),
        }
    }
}

use self::TokenType::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token(pub TokenType, pub String);

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Operator => write!(f, "operator: '{}'", self.1),
            Id => write!(f, "id: '{}'", self.1),
            Num => write!(f, "num: '{}'", self.1),
            NoteString => write!(f, "pitch: '{}'", self.1),
            Keyword => write!(f, "keyword: '{}'", self.1),
            Delimeter => write!(f, "delimeter: '{}'", self.1),
            BackLink => write!(f, "'!'"),
            Dot => write!(f, "'.'"),
            Rest => write!(f, "'_'"),
            Done => write!(f, "end of file"),
            Unknown => write!(f, "unknown"),
            Empty => write!(f, "empty"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeLocation {
    pub line: usize,
    pub column: usize,
    pub file: String,
}

impl fmt::Display for CodeLocation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

#[derive(Debug)]
pub struct Lexer {
    loc: CodeLocation,
    was_put_back: bool,
    c: [u8; 1],
    file: File,
}

impl Lexer {
    pub fn new(file: &str) -> SonnyResult<Lexer> {
        Ok(Lexer {
            loc: CodeLocation {
                line: 1,
                column: 0,
                file: PathBuf::from(file)
                    .file_name()
                    .expect("unable to get file name")
                    .to_str()
                    .expect("unable to convert file name to str")
                    .to_string(),
            },
            was_put_back: false,
            c: [0],
            file: if let Ok(f) = File::open(file) {
                f
            } else {
                return Err(Error::new(ErrorSpec::FileNotFound(file.to_string())));
            },
        })
    }
    pub fn loc(&self) -> CodeLocation {
        self.loc.clone()
    }
    fn get_char(&mut self) -> Option<char> {
        if self.was_put_back {
            self.was_put_back = false;
            Some(self.c[0] as char)
        } else {
            if self.file.read_exact(&mut self.c).is_ok() {
                self.loc.column += 1;
                Some(self.c[0] as char)
            } else {
                None
            }
        }
    }
    fn put_back(&mut self) {
        self.was_put_back = true;
    }
    pub fn lex(&mut self) -> Token {
        // Begin reading a token
        while let Some(c) = self.get_char() {
            let mut token = String::new();

            // Check for tokens that start with alpha or us
            if c.is_alphabetic() || c == '_' {
                token.push(c);
                // Check for ids, keywords, built_ins and notes
                while let Some(c) = self.get_char() {
                    if c.is_alphanumeric() || c == '_' {
                        token.push(c);
                    } else {
                        self.put_back();
                        // Check for pi
                        if token == "pi" {
                            return Token(Num, token);
                        }
                        // Check for keywords
                        if KEYWORDS.iter().find(|&k| k == &token).is_some() {
                            return Token(Keyword, token);
                        }
                        // Check for notes
                        let bytes: Vec<u8> = token.chars().map(|cc| cc as u8).collect();
                        let mut i = 0;
                        if bytes[i] >= 'A' as u8 && bytes[i] <= 'G' as u8 {
                            if bytes.len() == 1 {
                                return Token(NoteString, token);
                            } else {
                                i += 1;
                                if bytes[i] == 'b' as u8 || bytes[i] == '#' as u8 {
                                    if bytes.len() == 2 {
                                        return Token(NoteString, token);
                                    }
                                    i += 1;
                                }
                                while i < bytes.len() {
                                    if !(bytes[i] as char).is_digit(10) {
                                        return Token(Id, token);
                                    }
                                    i += 1;
                                }
                                return Token(NoteString, token);
                            }
                        }

                        return Token(Id, token);
                    }
                }
            }
            // Check for valid num tokens
            else if c.is_digit(10) {
                token.push(c);
                while let Some(c) = self.get_char() {
                    if c.is_digit(10) {
                        token.push(c);
                    } else {
                        self.put_back();
                        return Token(Num, token);
                    }
                }
            }
            // Check for newlines
            else if c.is_whitespace() {
                if c == '\n' {
                    self.loc.line += 1;
                    self.loc.column = 0;
                }
            }
            // Check for operators
            else {
                token.push(c);
                match c {
                    '(' | ')' | '{' | '}' | '[' | ']' | ',' => return Token(Delimeter, token),
                    '|' => if let Some(c) = self.get_char() {
                        if c == '|' {
                            token.push(c);
                            return Token(Operator, token);
                        } else {
                            self.put_back();
                            return Token(Delimeter, token);
                        }
                    },
                    '&' => if let Some(c) = self.get_char() {
                        if c == '&' {
                            token.push(c);
                            return Token(Operator, token);
                        } else {
                            self.put_back();
                            return Token(Unknown, token);
                        }
                    },
                    ':' => {
                        if let Some(c) = self.get_char() {
                            if c == ':' {
                                token.push(c);
                            } else {
                                self.put_back();
                            }
                            return Token(Delimeter, token);
                        }
                    }
                    '=' => if let Some(c) = self.get_char() {
                        if c == '=' {
                            token.push(c);
                            return Token(Operator, token);
                        } else {
                            self.put_back();
                            return Token(Unknown, token);
                        }
                    },
                    '!' => if let Some(c) = self.get_char() {
                        if c == '=' {
                            token.push(c);
                            return Token(Operator, token);
                        } else {
                            self.put_back();
                            return Token(BackLink, token);
                        }
                    },
                    '<' => if let Some(c) = self.get_char() {
                        if c == '=' {
                            token.push(c);
                            return Token(Operator, token);
                        } else {
                            self.put_back();
                            return Token(Operator, token);
                        }
                    },
                    '>' => if let Some(c) = self.get_char() {
                        if c == '=' {
                            token.push(c);
                            return Token(Operator, token);
                        } else {
                            self.put_back();
                            return Token(Operator, token);
                        }
                    },
                    '.' => return Token(Dot, token),
                    '_' => return Token(Rest, token),
                    '+' | '*' | '%' | '^' | '?' => return Token(Operator, token),
                    '-' => if let Some(c) = self.get_char() {
                        if c == '>' {
                            token.push(c);
                            return Token(Delimeter, token);
                        } else {
                            self.put_back();
                            return Token(Operator, token);
                        }
                    },
                    '/' => {
                        if let Some(c) = self.get_char() {
                            match c {
                                '/' => {
                                    while let Some(c) = self.get_char() {
                                        if c == '\n' {
                                            break;
                                        }
                                    }
                                }
                                _ => {
                                    self.put_back();
                                    return Token(Operator, token);
                                }
                            }
                        }
                    }
                    _ => {
                        token.push(c);
                        return Token(Unknown, token);
                    }
                }
            }
        }
        Token(Done, String::new())
    }
}
