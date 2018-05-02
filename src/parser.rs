use std::f64;

use either::*;

use builder::*;
use error::{ErrorSpec::*, *};
use lexer::TokenType::*;
use lexer::*;

fn string_to_pitch(s: &str) -> f64 {
    let bytes = s.as_bytes();
    let letter = bytes[0] as char;
    let mut octave = 3;
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
    local_offset += accidental;
    let offset = local_offset + (octave * 12) as i32;
    16.3516f64 * 1.059463094359f64.powf(offset as f64)
}

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    builder: Builder,
    look: Token,
    next: Token,
    peeked: bool,
    sample_rate: f64,
    curr_time: f64,
    paren_level: usize,
}

impl Parser {
    pub fn new(file: &str) -> SonnyResult<Parser> {
        let mut lexer = Lexer::new(file)?;
        let look = lexer.lex();
        Ok(Parser {
            lexer,
            builder: Builder::new(),
            look,
            next: Token(Empty, String::new()),
            peeked: false,
            sample_rate: 44100.0,
            curr_time: 0.0,
            paren_level: 0,
        })
    }
    pub fn parse(mut self) -> SonnyResult<Builder> {
        // Parse everything into chains
        while self.look.0 != Done {
            if self.look.1 == "tempo" {
                self.mas("tempo")?;
                self.mas(":")?;
                self.builder.tempo = self.real()?;
            }
            self.chain_declaration()?;
        }
        Ok(self.builder)
    }
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
    fn mas(&mut self, s: &str) -> SonnyResult<()> {
        if &self.look.1 == s {
            // println!("Expected {:?}, found {:?}", s, self.look.1.clone());
            if self.peeked {
                self.peeked = false;
                self.look = self.next.clone();
            } else {
                self.look = self.lexer.lex();
            }
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
    fn peek(&mut self) -> Token {
        if !self.peeked {
            self.peeked = true;
            self.next = self.lexer.lex();
        }
        self.next.clone()
    }
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
    fn pitch(&mut self) -> SonnyResult<f64> {
        Ok(if self.look.0 == NoteString {
            let pitch = string_to_pitch(&self.look.1.clone());
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
    fn dots(&mut self) -> SonnyResult<usize> {
        let mut result = 0;
        while self.look.1 == "." {
            self.mas(".")?;
            result += 1;
        }
        Ok(result)
    }
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
    fn note(&mut self) -> SonnyResult<Note> {
        let pitch = self.pitch()?;
        self.mas(":")?;
        let duration = self.duration()?;
        self.curr_time += duration;
        Ok(Note {
            pitch,
            period: Period {
                start: self.curr_time - duration,
                end: self.curr_time,
            },
        })
    }
    fn notes(&mut self) -> SonnyResult<Vec<Note>> {
        let mut note_list = Vec::new();
        note_list.push(self.note()?);
        while self.look.1 == "," {
            self.mas(",")?;
            note_list.push(self.note()?);
        }
        self.curr_time = 0.0;
        Ok(note_list)
    }
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
    fn term(&mut self) -> SonnyResult<Operand> {
        Ok(match self.look.0 {
            Num => Operand::Num(self.real()?),
            Keyword => {
                let op = match self.look.1.as_str() {
                    "time" => Operand::Time,
                    _ => return Err(Error::new(InvalidKeyword(self.look.1.clone())).on_line(self.lexer.loc())),
                };
                self.mat(Keyword)?;
                op
            }
            Id => {
                let id = self.look.1.clone();
                if !self.builder
                    .chains
                    .contains_key(&ChainName::String(id.clone()))
                {
                    return Err(Error::new(CantFindChain(ChainName::String(id.clone())))
                        .on_line(self.lexer.loc()));
                }
                self.mat(Id)?;
                if self.look.1 == "." {
                    self.mas(".")?;
                    if self.look.1 == "start" {
                        self.mas("start")?;
                        Operand::Property(ChainName::String(id), Property::Start)
                    } else if self.look.1 == "end" {
                        self.mas("end")?;
                        Operand::Property(ChainName::String(id), Property::End)
                    } else if self.look.1 == "dur" {
                        self.mas("dur")?;
                        Operand::Property(ChainName::String(id), Property::Duration)
                    } else {
                        return Err(Error::new(ExpectedNotesProperty(self.look.clone()))
                            .on_line(self.lexer.loc()));
                    }
                } else {
                    Operand::Id(ChainName::String(id))
                }
            }
            BackLink => self.backlink()?,
            Delimeter => {
                if self.look.1 == "(" {
                    self.mas("(")?;
                    let expr = self.expression()?;

                    self.mas(")")?;
                    Operand::Expression(Box::new(expr))
                } else {
                    return Err(Error::new(InvalidDelimeter(self.look.1.clone())).on_line(self.lexer.loc()));
                }
            }
            NoteString => {
                let note = Operand::Num(string_to_pitch(&self.look.1.clone()));
                self.mat(NoteString)?;
                note
            }
            Done => return Err(Error::new(UnexpectedEndOfFile).on_line(self.lexer.loc())),
            _ => return Err(Error::new(InvalidTerm(self.look.clone())).on_line(self.lexer.loc())),
        })
    }
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
        } else {
            Expression(Operation::Operand(self.term()?))
        })
    }
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
    fn exp_pow(&mut self) -> SonnyResult<Expression> {
        let mut expr = self.exp_min_max()?;
        loop {
            if self.look.1 == "^" {
                self.mas("^")?;
                expr = Expression(Operation::Power(
                    Operand::Expression(Box::new(expr)),
                    Operand::Expression(Box::new(self.exp_min_max()?)),
                ));
            } else if self.look.1 == "log" {
                self.mas("log")?;
                expr = Expression(Operation::Logarithm(Operand::Expression(Box::new(
                    self.exp_pow()?,
                ))));
            } else {
                break;
            }
        }
        Ok(expr)
    }
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
    fn expression(&mut self) -> SonnyResult<Expression> {
        self.exp_add()
    }
    fn link(&mut self) -> SonnyResult<()> {
        Ok(if self.look.1 == "|" {
            self.mas("|")?;
            let name = self.chain_declaration()?;
            self.mas("|")?;
            self.builder
                .new_expression(Expression(Operation::Operand(Operand::Id(name))))
        } else if self.look.1 == "{" {
            self.mas("{")?;
            let notes = self.notes()?;
            self.mas("}")?;
            self.builder
                .new_expression(Expression(Operation::Operand(Operand::Notes(notes))))
        } else {
            let expr = self.expression()?;
            self.builder.new_expression(expr);
        })
    }
    fn chain(&mut self) -> SonnyResult<()> {
        self.link()?;
        while self.look.1 == "->" {
            self.mas("->")?;
            if self.look.1 == "out" {
                self.builder.play_chain();
                self.mas("out")?;
                if self.look.1 == ":" {
                    self.mas(":")?;
                    self.builder.end_time = self.real()?;
                }
            } else {
                self.link()?;
            }
        }
        Ok(())
    }
    fn chain_declaration(&mut self) -> SonnyResult<ChainName> {
        self.builder.new_chain();
        let mut name = None;
        if self.look.0 == Id && self.peek().1 == ":" {
            name = Some(self.look.1.clone());
            self.mat(Id)?;
            self.mas(":")?;
        }
        self.chain()?;
        Ok(self.builder.finalize_chain(name))
    }
}
