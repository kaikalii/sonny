use std::cell::RefCell;
use std::io::Read;

static KEYWORDS: &[&'static str] = &["track", "env", "start", "end", "length", "time"];
static mut LINENO: u8 = 1;
static mut WAS_PUT_BACK: bool = false;
static mut C: char = 'x';

#[derive(Debug, Clone)]
pub enum TokenType {
    Operator,
    Id,
    Num,
    Note,
    Keyword,
    Delimeter,
    Misc,
    Done,
    Unknown,
}

use self::TokenType::*;

#[derive(Debug, Clone)]
pub struct Token {
    pub t: TokenType,
    pub s: String,
}

impl Token {
    pub fn new(t: TokenType, s: String) -> Token {
        Token { t, s }
    }
}

pub fn lexer(reader: &mut Read) -> Token {
    // Define local functions for reading characters from the reader
    let mut ch = [0u8];
    let mut get_char = || -> Option<char> {
        unsafe {
            if WAS_PUT_BACK {
                WAS_PUT_BACK = false;
                Some(C)
            } else {
                if reader.read_exact(&mut ch).is_ok() {
                    C = ch[0] as char;
                    Some(C)
                } else {
                    None
                }
            }
        }
    };
    let put_back = || unsafe { WAS_PUT_BACK = true };

    // Begin reading a token
    while let Some(c) = get_char() {
        let mut token = String::new();

        // Check for tokens that start with alpha or us
        if c.is_alphabetic() || c == '_' {
            token.push(c);
            // Check for ids, keywords, and notes
            while let Some(c) = get_char() {
                if c.is_alphanumeric() || c == '_' {
                    token.push(c);
                    // Check for keywords
                    if KEYWORDS.iter().find(|&k| k == &token).is_some() {
                        return Token::new(Keyword, token);
                    }
                } else {
                    put_back();
                    // Check for notes
                    let bytes: Vec<u8> = token.chars().map(|cc| cc as u8).collect();
                    let mut i = 0;
                    if bytes[i] >= 'A' as u8 && bytes[i] <= 'F' as u8 {
                        if bytes.len() == 1 {
                            return Token::new(Note, token);
                        } else {
                            i += 1;
                            if bytes[i] == 'b' as u8 || bytes[i] == '#' as u8 {
                                if bytes.len() == 2 {
                                    return Token::new(Note, token);
                                }
                                i += 1;
                            }
                            while i < bytes.len() {
                                if !(bytes[i] as char).is_digit(10) {
                                    return Token::new(Id, token);
                                }
                                i += 1;
                            }
                            return Token::new(Note, token);
                        }
                    }

                    return Token::new(Id, token);
                }
            }
        }
        // Check for valid num tokens
        else if c.is_digit(10) {
            token.push(c);
            while let Some(c) = get_char() {
                if c.is_digit(10) {
                    token.push(c);
                } else {
                    put_back();
                    return Token::new(Num, token);
                }
            }
        }
        // Check for newlines
        else if c.is_whitespace() {
            if c == '\n' {
                unsafe {
                    LINENO += 1;
                }
            }
        }
        // Check for operators
        else {
            token.push(c);
            match c {
                '(' | ')' | '{' | '}' | ',' => return Token::new(Delimeter, token),
                ':' => {
                    if let Some(c) = get_char() {
                        if c == ':' {
                            token.push(c);
                        } else {
                            put_back();
                        }
                        return Token::new(Delimeter, token);
                    }
                }
                '$' | '.' => return Token::new(Misc, token),
                '+' | '*' | '%' | '^' => return Token::new(Operator, token),
                '-' => {
                    if let Some(c) = get_char() {
                        if c == '>' {
                            token.push(c);
                            return Token::new(Delimeter, token);
                        } else {
                            put_back();
                            return Token::new(Operator, token);
                        }
                    }
                }
                '/' => {
                    if let Some(c) = get_char() {
                        match c {
                            '/' => {
                                while let Some(c) = get_char() {
                                    if c == '\n' {
                                        break;
                                    }
                                }
                            }
                            _ => {
                                put_back();
                                return Token::new(Operator, token);
                            }
                        }
                    }
                }
                _ => {
                    token.push(c);
                    return Token::new(Unknown, token);
                }
            }
        }
    }
    Token::new(Done, String::new())
}
