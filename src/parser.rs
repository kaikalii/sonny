use builder::*;
use lexer::TokenType::*;
use lexer::*;

fn string_to_pitch(s: &str) -> f64 {
    let bytes = s.as_bytes();
    let letter = bytes[0] as char;
    let mut octave = 3;
    let accidental = if bytes[1] as char == '#' {
        if s.len() == 2 {
            octave = bytes[2] as i8 - '0' as i8;
        }
        1
    } else if bytes[1] as char == 'b' {
        if s.len() == 2 {
            octave = bytes[2] as i8 - '0' as i8;
        }
        -1
    } else {
        octave = bytes[1] as i8 - '0' as i8;
        0
    };

    let mut local_offset = match letter {
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
    let offset = local_offset + octave * 12;
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
    tempo: f64,
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
            tempo: 120.0,
            curr_time: 0.0,
        }
    }
    pub fn parse(&mut self) {
        while self.look.0 != Done {
            self.builder.new_chain();
            let chain_name = self.chain_declaration();
            if let Some(cn) = chain_name {
                self.builder.name_chain(cn);
            }
        }
        println!("{:#?}", self.builder);
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
    fn match_many(&mut self, t: &[Token]) -> Token {
        if let Some(token) = t.iter().find(|&x| x == &self.look) {
            // println!(
            //     "Expected {:?}, found {:?}",
            //     t.iter().map(|x| x.1.clone()).collect::<Vec<String>>(),
            //     self.look.1
            // );
            if self.peeked {
                self.peeked = false;
                self.look = self.next.clone();
            } else {
                self.look = self.lexer.lex();
            }
            token.clone()
        } else {
            println!(
                "Expcted {:?} , found {:?} on line {}",
                t,
                self.look,
                self.lexer.lineno()
            );
            panic!("Bailing due to error.");
        }
    }
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
                (num1 / num2) / (self.tempo / 60.0)
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
        note_list
    }
    fn num_backlinks(&mut self) -> Vec<usize> {
        let mut num_links = Vec::new();
        if self.look.0 == Num {
            num_links.push(self.look.1.parse().expect(&format!(
                "Unable to parse numeric backlink on line {}",
                self.lexer.lineno(),
            )));
            self.mat(Num);
            while self.look.1 == "." {
                self.mas(".");
                num_links.push(self.look.1.parse().expect(&format!(
                    "Unable to parse numeric backlink on line {}",
                    self.lexer.lineno(),
                )));
                self.mat(Num);
            }
        }
        num_links
    }
    fn backlinks(&mut self) -> Operand {
        let mut id_link = None;
        let mut num_links = Vec::new();
        if self.look.0 == Id {
            id_link = Some(self.look.1.clone());
            self.mat(Id);
            if self.look.1 == "." {
                self.mas(".");
                num_links = self.num_backlinks();
            }
        } else {
            num_links = self.num_backlinks();
        }
        Operand::BackLink(id_link, num_links)
    }
    fn backlink(&mut self) -> Operand {
        self.mas("!");
        self.backlinks()
    }
    fn term(&mut self) -> Operand {
        match self.look.0 {
            Num => Operand::Num(self.real()),
            Keyword => {
                let op = match self.look.1.as_str() {
                    "time" => Operand::Time,
                    "sample_rate" => Operand::SampleRate,
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
                    let op = self.expression();
                    self.mas(")");
                    Operand::Operation(Box::new(op))
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
    fn exp_un(&mut self) -> Operation {
        if &self.look.1 == "-" {
            self.mas("-");
            Operation::Negate(self.term())
        } else if &self.look.1 == "sin" {
            self.mas("sin");
            Operation::Sine(self.term())
        } else if &self.look.1 == "cos" {
            self.mas("cos");
            Operation::Cosine(self.term())
        } else if &self.look.1 == "ceil" {
            self.mas("ceil");
            Operation::Ceiling(self.term())
        } else if &self.look.1 == "floor" {
            self.mas("floor");
            Operation::Floor(self.term())
        } else if &self.look.1 == "abs" {
            self.mas("abs");
            Operation::AbsoluteValue(self.term())
        } else {
            Operation::None(self.term())
        }
    }
    fn exp_pow(&mut self) -> Operation {
        let base_op = self.exp_un();
        if self.look.1 == "^" {
            self.mas("^");
            Operation::Power(
                Operand::Operation(Box::new(base_op)),
                Operand::Operation(Box::new(self.exp_pow())),
            )
        } else {
            base_op
        }
    }
    fn exp_mul(&mut self) -> Operation {
        let lhs_op = self.exp_pow();
        if self.look.1 == "*" {
            self.mas("*");
            Operation::Multiply(
                Operand::Operation(Box::new(lhs_op)),
                Operand::Operation(Box::new(self.exp_mul())),
            )
        } else if self.look.1 == "/" {
            self.mas("/");
            Operation::Divide(
                Operand::Operation(Box::new(lhs_op)),
                Operand::Operation(Box::new(self.exp_mul())),
            )
        } else if self.look.1 == "%" {
            self.mas("%");
            Operation::Remainder(
                Operand::Operation(Box::new(lhs_op)),
                Operand::Operation(Box::new(self.exp_mul())),
            )
        } else {
            lhs_op
        }
    }
    fn exp_add(&mut self) -> Operation {
        let lhs_op = self.exp_mul();
        if self.look.1 == "+" {
            self.mas("+");
            Operation::Add(
                Operand::Operation(Box::new(lhs_op)),
                Operand::Operation(Box::new(self.exp_add())),
            )
        } else if self.look.1 == "-" {
            self.mas("-");
            Operation::Substract(
                Operand::Operation(Box::new(lhs_op)),
                Operand::Operation(Box::new(self.exp_add())),
            )
        } else {
            lhs_op
        }
    }
    fn expression(&mut self) -> Operation {
        self.exp_add()
    }
    fn declaration_head(&mut self) -> (Option<String>, Period) {
        let p = self.peek();
        let mut name = None;
        let mut period = Period {
            start: Time::Start,
            end: Time::End,
        };
        if self.look.0 == Id && (p.1 == "[" || p.1 == ":") {
            name = Some(self.look.1.clone());
            self.mat(Id);
            if self.match_many(&[tok!(Delimeter, "["), tok!(Delimeter, ":")])
                .1
                .as_str() == "["
            {
                if self.look.1 == "start" {
                    self.mas("start");
                } else {
                    period.start = Time::Absolute(self.duration());
                }
                if self.look.1 == ":" {
                    self.mas(":");
                    if self.look.1 == "end" {
                        self.mas("end");
                    } else {
                        period.end = Time::Absolute(self.duration());
                    }
                }
                self.mas("]");
                self.mas(":");
            }
        }
        (name, period)
    }
    fn link(&mut self) {
        let (name, period) = self.declaration_head();
        if self.look.1 == "{" {
            self.mas("{");
            let note_list = self.notes();
            self.mas("}");
            self.builder.new_notes(name, period, note_list);
        } else {
            let expr_op = self.expression();
            self.builder.new_expression(name, period, expr_op);
        }
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
        if self.look.0 == Id && self.peek().1 == "::" {
            name = Some(self.look.1.clone());
            self.mat(Id);
            self.mas("::");
        }
        self.chain();
        name
    }
}
