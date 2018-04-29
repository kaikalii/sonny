use builder::*;
use lexer::TokenType::*;
use lexer::*;

fn string_to_pitch(s: &str) -> f64 {
    let bytes = s.as_bytes();
    let letter = bytes[0] as char;
    let mut octave = 3;
    let accidental: i32 = if bytes[1] as char == '#' {
        if s.len() == 3 {
            octave = (bytes[2] as char).to_digit(10).unwrap();
        }
        1
    } else if bytes[1] as char == 'b' {
        if s.len() == 3 {
            octave = (bytes[2] as char).to_digit(10).unwrap();
        }
        -1
    } else {
        octave = (bytes[1] as char).to_digit(10).unwrap();
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
}

impl Parser {
    pub fn new(file: &str) -> Parser {
        let mut lexer = Lexer::new(file);
        let look = lexer.lex();
        Parser {
            lexer,
            builder: Builder::new(),
            look,
            next: Token(Empty, String::new()),
            peeked: false,
            sample_rate: 44100.0,
            curr_time: 0.0,
        }
    }
    pub fn parse(mut self) -> Builder {
        while self.look.0 != Done {
            self.builder.new_chain();
            let chain_name = self.chain_declaration();
            self.builder.finalize_chain(chain_name);
        }
        self.builder
    }
    fn mat(&mut self, t: TokenType) {
        if self.look.0 == t {
            // println!("Expected {:?}, found {:?}", t, self.look.1);
            if self.peeked {
                self.peeked = false;
                self.look = self.next.clone();
            } else {
                self.look = self.lexer.lex();
            }
        } else {
            println!(
                "Unexpected {:?} : {:?} on line {}
                \n expected {:?}",
                self.look.0,
                self.look.1,
                self.lexer.lineno(),
                t,
            );
            panic!("Bailing due to error.");
        }
    }
    fn mas(&mut self, s: &str) {
        if &self.look.1 == s {
            // println!("Expected {:?}, found {:?}", s, self.look.1);
            if self.peeked {
                self.peeked = false;
                self.look = self.next.clone();
            } else {
                self.look = self.lexer.lex();
            }
        } else {
            println!(
                "Unexpected {:?} : {:?} on line {}\nexpected {:?}",
                self.look.0,
                self.look.1,
                self.lexer.lineno(),
                s,
            );
            panic!("Bailing due to error.");
        }
    }
    // fn match_many(&mut self, t: &[Token]) -> Token {
    //     if let Some(token) = t.iter().find(|&x| x == &self.look) {
    //         // println!(
    //         //     "Expected {:?}, found {:?}",
    //         //     t.iter().map(|x| x.1.clone()).collect::<Vec<String>>(),
    //         //     self.look.1
    //         // );
    //         if self.peeked {
    //             self.peeked = false;
    //             self.look = self.next.clone();
    //         } else {
    //             self.look = self.lexer.lex();
    //         }
    //         token.clone()
    //     } else {
    //         println!(
    //             "Expcted {:?} , found {:?} on line {}",
    //             t,
    //             self.look,
    //             self.lexer.lineno()
    //         );
    //         panic!("Bailing due to error.");
    //     }
    // }
    fn peek(&mut self) -> Token {
        if !self.peeked {
            self.peeked = true;
            self.next = self.lexer.lex();
        }
        self.next.clone()
    }
    fn real(&mut self) -> f64 {
        let mut num_str = String::new();
        if self.look.1 == "pi" {
            num_str.push_str("3.14159265358979323846");
            self.mas("pi");
        } else if self.look.0 == Num {
            num_str.push_str(&self.look.1);
            self.mat(Num);
            if self.look.1 == "." {
                num_str.push_str(&self.look.1);
                self.mas(".");
                if self.look.0 == Num {
                    num_str.push_str(&self.look.1);
                    self.mat(Num);
                }
            }
        } else if self.look.1 == "." && self.peek().0 == Num {
            num_str.push_str(&self.look.1);
            self.mas(".");
            if self.look.0 == Num {
                num_str.push_str(&self.look.1);
                self.mat(Num);
            }
        }
        num_str
            .parse::<f64>()
            .expect(&format!("Unable to parse real num string: {}", num_str))
    }
    fn pitch(&mut self) -> f64 {
        if self.look.0 == NoteString {
            let pitch = string_to_pitch(&self.look.1);
            self.mat(NoteString);
            pitch
        } else if self.look.0 == Num {
            self.real()
        } else {
            panic!(
                "Invalid pitch {:?} on line {}",
                self.look.1,
                self.lexer.lineno()
            );
        }
    }
    fn duration(&mut self) -> f64 {
        if self.look.0 == Num {
            if self.peek().1 == "/" {
                let num1 = self.look.1.parse::<f64>().expect(&format!(
                    "Unable to parse duration num {:?} on line {}",
                    self.look.1,
                    self.lexer.lineno(),
                ));
                self.mat(Num);
                self.mas("/");
                let num2 = self.look.1.parse::<f64>().expect(&format!(
                    "Unable to parse duration num {:?} on line {}",
                    self.look.1,
                    self.lexer.lineno(),
                ));
                self.mat(Num);
                (num1 / num2) / (self.builder.tempo / 60.0) * 4.0
            } else {
                self.real()
            }
        } else {
            panic!("Invalid duration on line {}", self.lexer.lineno());
        }
    }
    fn look_num_note(&self) -> bool {
        self.look.0 == NoteString || self.look.0 == Num
    }
    fn note(&mut self) -> Note {
        let pitch = self.pitch();
        self.mas(":");
        let duration = self.duration();
        self.curr_time += duration;
        Note {
            pitch,
            period: Period {
                start: Time::Absolute(self.curr_time - duration),
                end: Time::Absolute(self.curr_time),
            },
        }
    }
    fn notes(&mut self) -> Vec<Note> {
        let mut note_list = Vec::new();
        if self.look_num_note() {
            note_list.push(self.note());
            while self.look.1 == "," {
                self.mas(",");
                note_list.push(self.note());
            }
        }
        self.curr_time = 0.0;
        note_list
    }
    fn backlink(&mut self) -> Operand {
        self.mas("!");
        let op = Operand::BackLink(if let Ok(x) = self.look.1.parse() {
            x
        } else {
            panic!(
                "Invalid backlink number \"{}\" on line {}",
                self.look.1,
                self.lexer.lineno()
            )
        });
        self.mat(Num);
        op
    }
    fn term(&mut self) -> Operand {
        match self.look.0 {
            Num => Operand::Num(self.real()),
            Keyword => {
                let op = match self.look.1.as_str() {
                    "time" => Operand::Time,
                    _ => panic!(
                        "Keyword term {:?} is invalid on line {}",
                        self.look.1,
                        self.lexer.lineno()
                    ),
                };
                self.mat(Keyword);
                op
            }
            Id => {
                let id = self.look.1.clone();
                self.mat(Id);
                Operand::Id(id)
            }
            Misc => if self.look.1 == "!" {
                self.backlink()
            } else {
                panic!(
                    "Misc term {:?} is invalid on line {}",
                    self.look.1,
                    self.lexer.lineno()
                );
            },
            Delimeter => {
                if self.look.1 == "(" {
                    self.mas("(");
                    let expr = self.expression();
                    self.mas(")");
                    Operand::Expression(Box::new(expr))
                } else if self.look.1 == "{" {
                    self.mas("{");
                    let notes = self.notes();
                    self.mas("}");
                    Operand::Notes(notes)
                } else {
                    panic!("Invalid delimeter on line {}", self.lexer.lineno());
                }
            }
            _ => panic!(
                "Invalid term {:?} on line {}",
                self.look.1,
                self.lexer.lineno()
            ),
        }
    }
    fn exp_un(&mut self) -> Expression {
        let period = self.period();
        if &self.look.1 == "-" {
            self.mas("-");
            Expression::new(Operation::Negate(self.term()), period)
        } else if &self.look.1 == "sin" {
            self.mas("sin");
            Expression::new(Operation::Sine(self.term()), period)
        } else if &self.look.1 == "cos" {
            self.mas("cos");
            Expression::new(Operation::Cosine(self.term()), period)
        } else if &self.look.1 == "ceil" {
            self.mas("ceil");
            Expression::new(Operation::Ceiling(self.term()), period)
        } else if &self.look.1 == "floor" {
            self.mas("floor");
            Expression::new(Operation::Floor(self.term()), period)
        } else if &self.look.1 == "abs" {
            self.mas("abs");
            Expression::new(Operation::AbsoluteValue(self.term()), period)
        } else {
            Expression::new(Operation::Operand(self.term()), period)
        }
    }
    fn exp_pow(&mut self) -> Expression {
        let period = self.period();
        let base_op = self.exp_un();
        if self.look.1 == "^" {
            self.mas("^");
            Expression::new(
                Operation::Power(
                    Operand::Expression(Box::new(base_op)),
                    Operand::Expression(Box::new(self.exp_pow())),
                ),
                period,
            )
        } else {
            base_op
        }
    }
    fn exp_mul(&mut self) -> Expression {
        let period = self.period();
        let lhs_op = self.exp_pow();
        if self.look.1 == "*" {
            self.mas("*");
            Expression::new(
                Operation::Multiply(
                    Operand::Expression(Box::new(lhs_op)),
                    Operand::Expression(Box::new(self.exp_mul())),
                ),
                period,
            )
        } else if self.look.1 == "/" {
            self.mas("/");
            Expression::new(
                Operation::Divide(
                    Operand::Expression(Box::new(lhs_op)),
                    Operand::Expression(Box::new(self.exp_mul())),
                ),
                period,
            )
        } else if self.look.1 == "%" {
            self.mas("%");
            Expression::new(
                Operation::Remainder(
                    Operand::Expression(Box::new(lhs_op)),
                    Operand::Expression(Box::new(self.exp_mul())),
                ),
                period,
            )
        } else {
            lhs_op
        }
    }
    fn exp_add(&mut self) -> Expression {
        let period = self.period();
        let lhs_op = self.exp_mul();
        if self.look.1 == "+" {
            self.mas("+");
            Expression::new(
                Operation::Add(
                    Operand::Expression(Box::new(lhs_op)),
                    Operand::Expression(Box::new(self.exp_add())),
                ),
                period,
            )
        } else if self.look.1 == "-" {
            self.mas("-");
            Expression::new(
                Operation::Subtract(
                    Operand::Expression(Box::new(lhs_op)),
                    Operand::Expression(Box::new(self.exp_add())),
                ),
                period,
            )
        } else {
            lhs_op
        }
    }
    fn expression(&mut self) -> Expression {
        self.exp_add()
    }
    fn period(&mut self) -> Period {
        let mut period = Period {
            start: Time::Start,
            end: Time::End,
        };
        if self.look.1 == "[" {
            self.mas("[");
            if self.look.1 == "start" {
                self.mas("start");
            } else {
                period.start = Time::Absolute(self.duration());
            }
            self.mas(":");
            if self.look.1 == "end" {
                self.mas("end");
            } else {
                period.end = Time::Absolute(self.duration());
            }
            self.mas("]");
        }
        period
    }
    fn link(&mut self) {
        let expr = self.expression();
        self.builder.new_expression(expr);
    }
    fn chain(&mut self) {
        self.link();
        while self.look.1 == "->" {
            self.mas("->");
            if self.look.1 == "out" {
                self.builder.play_chain();
                self.mas("out")
            } else {
                self.link();
            }
        }
    }
    fn chain_declaration(&mut self) -> Option<String> {
        let mut name = None;
        if self.look.0 == Id && self.peek().1 == ":" {
            name = Some(self.look.1.clone());
            self.mat(Id);
            self.mas(":");
        }
        self.chain();
        name
    }
}
