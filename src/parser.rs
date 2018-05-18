use std::env;
use std::f64;
use std::path::PathBuf;

use either::*;

use find_folder::{Search, SearchFolder};

use builder::{variable::*, *};
use error::{ErrorSpec::*, *};
use lexer::TokenType::*;
use lexer::*;

// Parses tokens from the Lexer and invokes the Builder accordingly
#[derive(Debug)]
pub struct Parser {
    // The name of top-level file being compiled
    main_file_name: String,
    // The lexical analyzer
    lexer: Lexer,
    // The chain builder
    builder: Builder,
    // The next token to be parsed
    look: Token,
    // The next next token to be parsed
    next: Token,
    // Wheter or not peek() has recently been called
    peeked: bool,
    // The sample rate of the output
    sample_rate: f64,
    // The current time. Used for correctly assigned periods to notes
    curr_time: f64,
    // How many levels deep of parenthesis the parser is
    paren_level: usize,
    // The last octave used by notes
    last_note_octave: usize,
}

impl Parser {
    // Creates a new Parser which will parse the given file and
    // invoke the given builder
    pub fn new(file: &str, builder: Builder) -> SonnyResult<Parser> {
        let mut lexer = Lexer::new(file)?;
        let look = lexer.lex();
        Ok(Parser {
            main_file_name: file.to_string(),
            lexer,
            builder,
            look,
            next: Token(Empty, String::new()),
            peeked: false,
            sample_rate: 44100.0,
            curr_time: 0.0,
            paren_level: 0,
            last_note_octave: 3,
        })
    }
    // Parse the whole file and return the builder so that it can be
    // used by higher-level parsers or the write function itself.
    pub fn parse(mut self, finalize: bool) -> SonnyResult<Builder> {
        // Determine the name of this file's top-level chain
        let top_chain_name = PathBuf::from(self.lexer.loc().file)
            .file_stem()
            .expect("Unable to get file stem from file path")
            .to_str()
            .expect("Unable to convert file steam to &str")
            .to_string();
        // If this chain name already exists, then this is a file that
        // has already been included. Return the builder.
        if self.builder
            .find_chain(&ChainName::Scoped(top_chain_name.clone()))
            .is_some()
        {
            return Ok(self.builder);
        }
        // Create this file's top-level chain
        self.builder
            .new_chain(Some(top_chain_name), self.lexer.loc())?;
        // While in this file
        while self.look.0 != Done {
            // Check for tempo setting
            if self.look.1 == "tempo" {
                self.mas("tempo")?;
                self.mas(":")?;
                self.builder.tempo = self.real()?;
            }
            // check for "include" keyword
            else if self.look.1 == "std" || self.look.1 == "include" {
                let standard = self.look.1 == "std";
                if standard {
                    self.mas("std")?;
                } else {
                    self.mas("include")?;
                }
                let mut filename = self.look.1.clone();
                self.mat(Id)?;
                while self.look.1 == "::" {
                    self.mas("::")?;
                    filename.push_str("/");
                    filename.push_str(&self.look.1);
                    self.mat(Id)?;
                }
                filename.push_str(".son");
                // Temporarily pop off this file's scope
                let temp_scope = self.builder
                    .names_in_scope
                    .pop()
                    .expect("no chains in scope");

                // Create the new file path
                let path = if standard {
                    SearchFolder {
                        start: PathBuf::from(
                            env::current_exe()
                                .expect("Unable to determine sonny executable path")
                                .parent()
                                .expect("Unable to get sonny executable parent"),
                        ),
                        direction: Search::ParentsThenKids(3, 3),
                    }.for_folder("std")
                        .expect("Unable to find standard library folder")
                        .join(filename)
                } else {
                    PathBuf::from(&self.main_file_name)
                        .parent()
                        .expect("Unable to get main file parent")
                        .join(filename.clone())
                };

                // Create a new parser for the file. Give it this parser's builder.
                // It's okay. It will get the builder back when the other parser
                // is done.
                self.builder = Parser::new(
                    &path.to_str().expect("unable to convert path to string"),
                    self.builder,
                )?.parse(true)?;

                // Put back the popped file scope.
                self.builder.names_in_scope.push(temp_scope);
            }
            // check for "use" keyword
            else if self.look.1 == "use" {
                self.mas("use")?;
                let mut name = self.look.1.clone();
                self.mat(Id)?;
                let mut broke = false;
                while self.look.1 == "::" {
                    self.mas("::")?;
                    if self.look.0 == Id {
                        name.push_str(&format!("::{}", self.look.1));
                        self.mat(Id)?;
                    } else if self.look.1 == "*" {
                        self.mas("*")?;
                        self.builder.names_in_scope.insert(
                            0,
                            NameInScope {
                                name: name.clone(),
                                contents: true,
                            },
                        );
                        broke = true;
                        break;
                    }
                }
                if !broke {
                    self.builder.names_in_scope.insert(
                        0,
                        NameInScope {
                            name,
                            contents: false,
                        },
                    );
                }
            } else {
                // Declare a chain
                self.chain_declaration()?;
            }
        }
        // If this is not the top-level parser, finalize the top-level file chain
        if finalize {
            self.builder.finalize_chain();
        }
        Ok(self.builder)
    }
    // Match the next token against a token type.
    // Used when the desired match is something generic,
    // link an Id or a Num
    fn mat(&mut self, t: TokenType) -> SonnyResult<()> {
        if self.look.0 == t {
            // println!("Expected {:?}, found {:?}", t, self.look.1.clone());
            Ok(if self.peeked {
                self.peeked = false;
                self.look = self.next.clone();
            } else {
                self.look = self.lexer.lex();
            })
        } else {
            Err(
                Error::new(ErrorSpec::ExpectedFound(Left(t), self.look.clone()))
                    .on_line(self.lexer.loc()),
            )
        }
    }
    // Match the next token agaist a &str.
    // Used when the desired match is a specific string.
    // This and mat() should probably be combined into a single macro.
    fn mas(&mut self, s: &str) -> SonnyResult<()> {
        if &self.look.1 == s {
            // println!("Expected {:?}, found {:?}", s, self.look.1.clone());
            if self.peeked {
                self.peeked = false;
                self.look = self.next.clone();
            } else {
                self.look = self.lexer.lex();
            }
            // This is probably not where this paren check should go.
            Ok(if s == "(" {
                self.paren_level += 1;
            } else if s == ")" {
                if self.paren_level > 0 {
                    self.paren_level -= 1;
                } else {
                    return Err(Error::new(ErrorSpec::CloseDelimeter(")".to_string()))
                        .on_line(self.lexer.loc()));
                }
            })
        } else {
            Err(Error::new(ErrorSpec::ExpectedFound(
                Right(s.to_string()),
                self.look.clone(),
            )).on_line(self.lexer.loc()))
        }
    }
    // Look at the token after the next one without consuming
    // the next one.
    fn peek(&mut self) -> Token {
        if !self.peeked {
            self.peeked = true;
            self.next = self.lexer.lex();
        }
        self.next.clone()
    }
    // Match a real number
    fn real(&mut self) -> SonnyResult<f64> {
        let mut num_str = String::new();
        if self.look.1 == "pi" {
            num_str.push_str("3.14159265358979323846");
            self.mas("pi")?;
        } else if self.look.0 == Num {
            num_str.push_str(&self.look.1.clone());
            self.mat(Num)?;
            if self.look.1 == "." {
                num_str.push_str(&self.look.1.clone());
                self.mas(".")?;
                if self.look.0 == Num {
                    num_str.push_str(&self.look.1.clone());
                    self.mat(Num)?;
                }
            }
        } else if self.look.1 == "." && self.peek().0 == Num {
            num_str.push_str(&self.look.1.clone());
            self.mas(".")?;
            if self.look.0 == Num {
                num_str.push_str(&self.look.1.clone());
                self.mat(Num)?;
            }
        }
        Ok(num_str
            .parse::<f64>()
            .expect(&format!("Unable to parse real num string: {}", num_str)))
    }
    // Convert a string representing a pitch into a number
    fn string_to_pitch(&mut self, s: &str) -> f64 {
        let bytes = s.as_bytes();
        let letter = bytes[0] as char;
        let mut octave = self.last_note_octave as u32;
        let accidental: i32 = if bytes.len() > 1 {
            if bytes[1] as char == '#' {
                if s.len() == 3 {
                    octave = (bytes[2] as char)
                        .to_digit(10)
                        .expect("unable to convert char to digit");
                }
                1
            } else if bytes[1] as char == 'b' {
                if s.len() == 3 {
                    octave = (bytes[2] as char)
                        .to_digit(10)
                        .expect("unable to convert char to digit");
                }
                -1
            } else {
                if s.len() == 2 {
                    octave = (bytes[1] as char)
                        .to_digit(10)
                        .expect("unable to convert char to digit");
                }
                0
            }
        } else {
            0
        };

        let mut local_offset: i32 = match letter {
            'C' => 0,
            'D' => 2,
            'E' => 4,
            'F' => 5,
            'G' => 7,
            'A' => 9,
            'B' => 11,
            _ => panic!("Invalid note letter"),
        };
        self.last_note_octave = octave as usize;
        local_offset += accidental;
        let offset = local_offset + (octave * 12) as i32;
        16.3516f64 * 1.059463094359f64.powf(offset as f64)
    }
    // Match a pitch element
    fn pitch_element(&mut self) -> SonnyResult<f64> {
        Ok(if self.look.0 == NoteString {
            let note_string = self.look.1.clone();
            let pitch = self.string_to_pitch(&note_string);
            self.mat(NoteString)?;
            pitch
        } else if self.look.0 == Num {
            self.real()?
        } else if self.look.1 == "_" {
            self.mas("_")?;
            0.0
        } else {
            return Err(Error::new(InvalidPitch(self.look.clone())).on_line(self.lexer.loc()));
        })
    }
    // Match a list of pitches
    fn pitch_list(&mut self) -> SonnyResult<Vec<f64>> {
        let mut result = Vec::new();
        if self.look.1 != "]" {
            result.push(self.pitch_element()?);
            while self.look.1 == "," {
                self.mas(",")?;
                if self.look.1 == "]" {
                    break;
                }
                result.push(self.pitch_element()?);
            }
        }
        Ok(result)
    }
    // Match a pitch
    fn pitch(&mut self) -> SonnyResult<Vec<f64>> {
        if self.look.1 == "[" {
            self.mas("[")?;
            let list = self.pitch_list()?;
            self.mas("]")?;
            Ok(list)
        } else {
            Ok(vec![self.pitch_element()?])
        }
    }
    // Match a sequence of '.'s
    fn dots(&mut self) -> SonnyResult<usize> {
        let mut result = 0;
        while self.look.1 == "." {
            self.mas(".")?;
            result += 1;
        }
        Ok(result)
    }
    // Match a duration
    fn duration(&mut self) -> SonnyResult<f64> {
        Ok(if self.look.0 == Num {
            if self.peek().1 == "/" {
                let num1 = self.look.1.parse::<f64>().expect(&format!(
                    "Unable to parse duration num {:?} on line {:?}",
                    self.look.1,
                    self.lexer.loc(),
                ));
                self.mat(Num)?;
                self.mas("/")?;
                let num2 = self.look.1.parse::<f64>().expect(&format!(
                    "Unable to parse duration num {:?} on line {:?}",
                    self.look.1,
                    self.lexer.loc(),
                ));
                self.mat(Num)?;
                (num1 / num2) / (self.builder.tempo / 60.0) * 4.0
            } else {
                self.real()?
            }
        } else {
            let mut frac = match self.look.1.as_ref() {
                "w" => 1.0,
                "h" => 0.5,
                "q" => 0.25,
                "e" => 0.125,
                "s" => 0.0625,
                "ts" => 0.03125,
                _ => {
                    return Err(Error::new(DurationQuantifier(self.look.clone())).on_line(self.lexer.loc()))
                }
            } / (self.builder.tempo / 60.0) * 4.0;
            self.mat(Keyword)?;
            for i in 0..self.dots()? {
                frac += frac / 2usize.pow(i as u32 + 1) as f64;
            }
            frac
        })
    }
    // Match a note which has both pitch and duration
    fn note(&mut self) -> SonnyResult<Note> {
        let pitch = self.pitch()?;
        self.mas(":")?;
        let duration = if self.look.0 == Id {
            let possible_chain_name = ChainName::Scoped(self.look.1.clone());
            self.mat(Id)?;
            if let Some(ref chain) = self.builder.find_chain(&possible_chain_name) {
                if let ChainLinks::OnlyNotes(ref _notes_or_ids, period) = chain.links {
                    period.duration()
                } else {
                    return Err(Error::new(DurationOfGenericChain(possible_chain_name))
                        .on_line(self.lexer.loc()));
                }
            } else {
                return Err(Error::new(CantFindChain(possible_chain_name)).on_line(self.lexer.loc()));
            }
        } else {
            self.duration()?
        };
        self.curr_time += duration;
        Ok(Note {
            pitches: pitch,
            period: Period {
                start: self.curr_time - duration,
                end: self.curr_time,
            },
        })
    }
    // Match a series of ,-separated notes
    fn notes(&mut self) -> SonnyResult<Vec<Note>> {
        let mut note_list = Vec::new();
        note_list.push(self.note()?);
        while self.look.1 == "," {
            self.mas(",")?;
            note_list.push(self.note()?);
        }
        self.last_note_octave = 3;
        self.curr_time = 0.0;
        Ok(note_list)
    }
    // Match a backlink
    fn backlink(&mut self) -> SonnyResult<Operand> {
        self.mas("!")?;
        let op = Operand::BackLink(if let Ok(x) = self.look.1.parse() {
            if x == 0 {
                return Err(Error::new(ZeroBacklink).on_line(self.lexer.loc()));
            } else {
                x
            }
        } else {
            return Err(Error::new(InvalidBackLink(self.look.clone())).on_line(self.lexer.loc()));
        });
        self.mat(Num)?;
        Ok(op)
    }
    // Match an indexer
    fn indexer(&mut self) -> SonnyResult<(Option<Expression>, Option<Expression>)> {
        if self.look.1 == "[" {
            self.mas("[")?;
            let start_expr = self.expression()?;
            if self.look.1 == ".." {
                self.mas("..")?;
                let end_expr = self.expression()?;
                self.mas("]")?;
                Ok((Some(start_expr), Some(end_expr)))
            } else {
                self.mas("]")?;
                Ok((Some(start_expr), None))
            }
        } else {
            Ok((None, None))
        }
    }
    // Match a list of expressions
    fn expression_list(&mut self) -> SonnyResult<Operand> {
        let mut result = Vec::new();
        if self.look.1 != "]" {
            result.push(self.expression()?);
            while self.look.1 == "," {
                self.mas(",")?;
                if self.look.1 == "]" {
                    break;
                }
                result.push(self.expression()?);
            }
        }
        Ok(Operand::Array(result))
    }
    // Match an expression term identifier
    fn term_identifier(&mut self) -> SonnyResult<Operand> {
        match self.look.0 {
            Num => Ok(Operand::Var(Variable::Number(self.real()?))),
            Keyword => {
                let op = match self.look.1.as_str() {
                    "time" => Operand::Time,
                    "window_size" => Operand::WindowSize,
                    "sample_rate" => Operand::SampleRate,
                    _ => return Err(Error::new(InvalidKeyword(self.look.1.clone())).on_line(self.lexer.loc())),
                };
                self.mat(Keyword)?;
                Ok(op)
            }
            Id => {
                let mut name = ChainName::Scoped(self.look.1.clone());
                self.mat(Id)?;
                while self.look.1 == "::" {
                    self.mas("::")?;
                    let next_id = self.look.1.clone();
                    self.mat(Id)?;
                    if let ChainName::Scoped(ref mut name) = name {
                        name.push_str("::");
                        name.push_str(&next_id);
                    }
                }
                match self.builder.find_chain(&name) {
                    Some(ref chain) => name = chain.name.clone(),
                    None => return Err(Error::new(CantFindChain(name)).on_line(self.lexer.loc())),
                }
                if self.look.1 == "." {
                    self.mas(".")?;
                    let property_name = self.look.1.clone();
                    let operand = if self.look.1 == "start" {
                        self.mas("start")?;
                        Ok(Operand::Property(name.clone(), Property::Start))
                    } else if self.look.1 == "end" {
                        self.mas("end")?;
                        Ok(Operand::Property(name.clone(), Property::End))
                    } else if self.look.1 == "dur" {
                        self.mas("dur")?;
                        Ok(Operand::Property(name.clone(), Property::Duration))
                    } else if self.look.1 == "prop" {
                        self.mas("prop")?;
                        Ok(Operand::Property(name.clone(), Property::All))
                    } else {
                        Err(Error::new(ExpectedNotesProperty(self.look.clone()))
                            .on_line(self.lexer.loc()))
                    };
                    if let ChainLinks::Generic(..) = self.builder
                        .find_chain(&name)
                        .expect("Unable to find chain")
                        .links
                    {
                        return Err(Error::new(PropertyOfGenericChain(name, property_name))
                            .on_line(self.lexer.loc()));
                    }
                    operand
                } else {
                    Ok(Operand::Id(name))
                }
            }
            BackLink => Ok(self.backlink()?),
            Delimeter => {
                if self.look.1 == "(" {
                    self.mas("(")?;
                    let expr = self.expression()?;

                    self.mas(")")?;
                    Ok(Operand::Expression(Box::new(expr)))
                } else if self.look.1 == "|" {
                    self.mas("|")?;
                    let name = self.chain_declaration()?;
                    self.mas("|")?;
                    Ok(Operand::Id(name))
                } else if self.look.1 == "[" {
                    self.mas("[")?;
                    let list = self.expression_list()?;
                    self.mas("]")?;
                    Ok(list)
                } else {
                    return Err(Error::new(InvalidDelimeter(self.look.1.clone())).on_line(self.lexer.loc()));
                }
            }
            NoteString => {
                let note_string = self.look.1.clone();
                let note = Operand::Var(Variable::Number(self.string_to_pitch(&note_string)));
                self.mat(NoteString)?;
                Ok(note)
            }
            Done => return Err(Error::new(UnexpectedEndOfFile).on_line(self.lexer.loc())),
            _ => return Err(Error::new(InvalidTerm(self.look.clone())).on_line(self.lexer.loc())),
        }
    }
    // Match an expression term
    fn term(&mut self) -> SonnyResult<Expression> {
        let ident = self.term_identifier()?;
        let index = self.indexer()?;
        if let Some(start) = index.0 {
            if let Some(end) = index.1 {
                Ok(Expression(Operation::SubArray(
                    ident,
                    Operand::Expression(Box::new(start)),
                    Operand::Expression(Box::new(end)),
                )))
            } else {
                Ok(Expression(Operation::Index(
                    ident,
                    Operand::Expression(Box::new(start)),
                )))
            }
        } else {
            Ok(Expression(Operation::Operand(ident)))
        }
    }
    // Match a unary expression
    fn exp_un(&mut self) -> SonnyResult<Expression> {
        Ok(if &self.look.1 == "-" {
            self.mas("-")?;
            Expression(Operation::Negate(Operand::Expression(Box::new(
                self.exp_un()?,
            ))))
        } else if &self.look.1 == "sin" {
            self.mas("sin")?;
            Expression(Operation::Sine(Operand::Expression(Box::new(
                self.exp_un()?,
            ))))
        } else if &self.look.1 == "cos" {
            self.mas("cos")?;
            Expression(Operation::Cosine(Operand::Expression(Box::new(
                self.exp_un()?,
            ))))
        } else if &self.look.1 == "ceil" {
            self.mas("ceil")?;
            Expression(Operation::Ceiling(Operand::Expression(Box::new(
                self.exp_un()?,
            ))))
        } else if &self.look.1 == "floor" {
            self.mas("floor")?;
            Expression(Operation::Floor(Operand::Expression(Box::new(
                self.exp_un()?,
            ))))
        } else if &self.look.1 == "abs" {
            self.mas("abs")?;
            Expression(Operation::AbsoluteValue(Operand::Expression(Box::new(
                self.exp_un()?,
            ))))
        } else if &self.look.1 == "avg" {
            self.mas("avg")?;
            Expression(Operation::Average(Operand::Expression(Box::new(
                self.exp_un()?,
            ))))
        } else {
            self.term()?
        })
    }
    // Match a min/max expression
    fn exp_min_max(&mut self) -> SonnyResult<Expression> {
        let mut expr = self.exp_un()?;
        loop {
            if self.look.1 == "min" {
                self.mas("min")?;
                expr = Expression(Operation::Min(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_un()?)),
                ));
            } else if self.look.1 == "max" {
                self.mas("max")?;
                expr = Expression(Operation::Max(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_un()?)),
                ));
            } else {
                break;
            }
        }
        Ok(expr)
    }
    // Match a power or logarithm expression
    fn exp_pow(&mut self) -> SonnyResult<Expression> {
        let mut expr;
        if self.look.1 == "log" {
            self.mas("log")?;
            expr = Expression(Operation::Logarithm(Operand::Expression(Box::new(
                self.exp_pow()?,
            ))));
        } else {
            expr = self.exp_min_max()?;
            loop {
                if self.look.1 == "^" {
                    self.mas("^")?;
                    expr = Expression(Operation::Power(
                        Operand::Expression(Box::new(expr)),
                        Operand::Expression(Box::new(self.exp_min_max()?)),
                    ));
                } else {
                    break;
                }
            }
        }
        Ok(expr)
    }
    // Match a multiplication, division, or remainder expression
    fn exp_mul(&mut self) -> SonnyResult<Expression> {
        let mut expr = self.exp_pow()?;
        loop {
            if self.look.1 == "*" {
                self.mas("*")?;
                expr = Expression(Operation::Multiply(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_pow()?)),
                ));
            } else if self.look.1 == "/" {
                self.mas("/")?;
                expr = Expression(Operation::Divide(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_pow()?)),
                ));
            } else if self.look.1 == "%" {
                self.mas("%")?;
                expr = Expression(Operation::Remainder(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_pow()?)),
                ));
            } else {
                break;
            }
        }
        Ok(expr)
    }
    // Match an addition or subtraction expression
    fn exp_add(&mut self) -> SonnyResult<Expression> {
        let mut expr = self.exp_mul()?;
        loop {
            if self.look.1 == "+" {
                self.mas("+")?;
                expr = Expression(Operation::Add(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_mul()?)),
                ));
            } else if self.look.1 == "-" {
                self.mas("-")?;
                expr = Expression(Operation::Subtract(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_mul()?)),
                ));
            } else {
                break;
            }
        }
        Ok(expr)
    }
    // Match a comparison expression
    fn exp_cmp(&mut self) -> SonnyResult<Expression> {
        let mut expr = self.exp_add()?;
        loop {
            if self.look.1 == "==" {
                self.mas("==")?;
                expr = Expression(Operation::Equal(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_add()?)),
                ));
            } else if self.look.1 == "!=" {
                self.mas("!=")?;
                expr = Expression(Operation::NotEqual(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_add()?)),
                ));
            } else if self.look.1 == "<" {
                self.mas("<")?;
                expr = Expression(Operation::LessThan(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_add()?)),
                ));
            } else if self.look.1 == ">" {
                self.mas(">")?;
                expr = Expression(Operation::GreaterThan(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_add()?)),
                ));
            } else if self.look.1 == "<=" {
                self.mas("<=")?;
                expr = Expression(Operation::LessThanOrEqual(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_add()?)),
                ));
            } else if self.look.1 == ">=" {
                self.mas(">=")?;
                expr = Expression(Operation::GreaterThanOrEqual(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_add()?)),
                ));
            } else {
                break;
            }
        }
        Ok(expr)
    }
    // Match an OR expression
    fn exp_or(&mut self) -> SonnyResult<Expression> {
        let mut expr = self.exp_cmp()?;
        if self.look.1 == "||" {
            self.mas("||")?;
            expr = Expression(Operation::Or(
                Operand::Expression(Box::new(expr)),
                Operand::Expression(Box::new(self.exp_or()?)),
            ));
        }
        Ok(expr)
    }
    // Match an AND expression
    fn exp_and(&mut self) -> SonnyResult<Expression> {
        let mut expr = self.exp_or()?;
        if self.look.1 == "&&" {
            self.mas("&&")?;
            expr = Expression(Operation::And(
                Operand::Expression(Box::new(expr)),
                Operand::Expression(Box::new(self.exp_and()?)),
            ));
        }
        Ok(expr)
    }
    // Match a ternary expression
    fn exp_tern(&mut self) -> SonnyResult<Expression> {
        let mut expr = self.exp_and()?;
        if self.look.1 == "?" {
            self.mas("?")?;
            expr = Expression(Operation::Ternary(
                Operand::Expression(Box::new(expr)),
                Operand::Expression({ Box::new(self.exp_tern()?) }),
                Operand::Expression({
                    self.mas(":")?;
                    Box::new(self.exp_tern()?)
                }),
            ));
        }
        Ok(expr)
    }
    // Match an entire expression
    fn expression(&mut self) -> SonnyResult<Expression> {
        self.exp_tern()
    }
    // Match a chain link
    fn link(&mut self) -> SonnyResult<()> {
        Ok(
            // Check for notes
            if self.look.1 == "{" {
                self.mas("{")?;
                let notes = self.notes()?;
                self.mas("}")?;
                self.builder
                    .new_expression(Expression(Operation::Operand(Operand::Notes(notes))))
            // It's an expression otherwise
            } else {
                let expr = self.expression()?;
                self.builder.new_expression(expr);
            },
        )
    }
    // Match the body of a chain
    fn chain(&mut self) -> SonnyResult<()> {
        self.link()?;
        while self.look.1 == "->" {
            self.mas("->")?;
            if self.look.1 == "out" {
                if self.builder.out_declared.is_none() {
                    self.builder.play_chain();
                    self.builder.out_declared = Some(self.lexer.loc());
                    self.mas("out")?;
                    if self.look.1 == ":" {
                        self.mas(":")?;
                        self.builder.end_time = self.real()?;
                    }
                    break;
                } else {
                    return Err(Error::new(MultipleOutChains(
                        self.builder.out_declared.clone().unwrap(),
                    )).on_line(self.lexer.loc()));
                }
            } else {
                self.link()?;
            }
        }
        Ok(())
    }
    // Match an entire chain, including the optional name
    fn chain_declaration(&mut self) -> SonnyResult<ChainName> {
        let mut name = None;
        if self.look.0 == Id && self.peek().1 == ":" {
            name = Some(self.look.1.clone());
            self.mat(Id)?;
            self.mas(":")?;
        }
        let chain_name = self.builder.new_chain(name, self.lexer.loc())?;
        self.chain()?;
        self.builder.finalize_chain();
        Ok(chain_name)
    }
}
