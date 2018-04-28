use lexer::TokenType::*;
use lexer::*;

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    look: Token,
    next: Token,
    peeked: bool,
    stop: u32,
}

impl Parser {
    pub fn new(file: &str) -> Parser {
        let mut lexer = Lexer::new(file);
        let look = lexer.lex();
        Parser {
            lexer,
            look,
            next: Token(Empty, String::new()),
            peeked: false,
            stop: 0,
        }
    }
    pub fn parse(&mut self) {
        while self.look.0 != Done {
            self.chain_declaration();
        }
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
    fn real(&mut self) {
        if self.look.0 == Num {
            self.mat(Num);
            if self.look.1 == "." {
                self.mas(".");
                if self.look.0 == Num {
                    self.mat(Num);
                }
            }
        } else if self.look.1 == "." {
            self.mas(".");
            if self.look.0 == Num {
                self.mat(Num);
            }
        } else if self.look.1 == "pi" {
            self.mas("pi");
        }
    }
    fn pitch(&mut self) {
        if self.look.0 == Note {
            self.mat(Note);
        } else if self.look.0 == Num {
            self.real();
        }
    }
    fn duration(&mut self) {
        if self.look.0 == Num {
            if self.peek().1 == "/" {
                self.mat(Num);
                self.mas("/");
                self.mat(Num);
            } else {
                self.real();
            }
        }
    }
    fn look_num_note(&self) -> bool {
        self.look.0 == Note || self.look.0 == Num
    }
    fn note(&mut self) {
        if self.look_num_note() {
            self.pitch();
            self.mas(":");
            self.duration();
        }
    }
    fn notes(&mut self) {
        if self.look_num_note() {
            self.note();
            while self.look.1 == "," {
                self.mas(",");
                self.note();
            }
        }
    }
    fn num_backlinks(&mut self) {
        if self.look.0 == Num {
            self.mat(Num);
            while self.look.1 == "." {
                self.mas(".");
                self.mat(Num);
            }
        }
    }
    fn backlinks(&mut self) {
        if self.look.0 == Id {
            self.mat(Id);
            if self.look.1 == "." {
                self.mas(".");
                self.num_backlinks();
            }
        } else {
            self.num_backlinks();
        }
    }
    fn backlink(&mut self) {
        self.mas("!");
        self.backlinks();
    }
    fn term(&mut self) {
        match self.look.0 {
            Num => self.real(),
            Keyword => self.mat(Keyword),
            Id => self.mat(Id),
            Misc => if self.look.1 == "!" {
                self.backlink();
            },
            Delimeter => {
                if self.look.1 == "(" {
                    self.mas("(");
                    self.expression();
                    self.mas(")");
                }
            }
            _ => (),
        }
    }
    fn exp_un(&mut self) {
        if &self.look.1 == "-" {
            self.mas("-");
        } else if &self.look.1 == "sin" {
            self.mas("sin");
        } else if &self.look.1 == "cos" {
            self.mas("cos");
        } else if &self.look.1 == "ceil" {
            self.mas("ceil");
        } else if &self.look.1 == "floor" {
            self.mas("floor");
        } else if &self.look.1 == "abs" {
            self.mas("abs");
        }
        self.term();
    }
    fn exp_pow(&mut self) {
        self.exp_un();
        while self.look.1 == "^" {
            self.mas("^");
            self.exp_un();
        }
    }
    fn exp_mul(&mut self) {
        self.exp_pow();
        if self.look.1 == "*" {
            self.mas("*");
            self.exp_mul();
        }
        if self.look.1 == "/" {
            self.mas("/");
            self.exp_mul();
        }
        if self.look.1 == "%" {
            self.mas("%");
            self.exp_mul();
        }
    }
    fn exp_add(&mut self) {
        self.exp_mul();
        if self.look.1 == "+" {
            self.mas("+");
            self.exp_add();
        }
        if self.look.1 == "-" {
            self.mas("-");
            self.exp_add();
        }
    }
    fn expression(&mut self) {
        // println!("start expression");
        self.exp_add();
        // println!("end expression");
    }
    fn declaration_head(&mut self) {
        let p = self.peek();
        if self.look.0 == Id && (p.1 == "[" || p.1 == ":") {
            // println!("start declaration_head");
            self.mat(Id);
            if self.match_many(&[tok!(Delimeter, "["), tok!(Delimeter, ":")])
                .1
                .as_str() == "["
            {
                self.duration();
                if self.look.1 == ":" {
                    self.mas(":");
                    if self.look.1 == "end" {
                        self.mas("end");
                    } else {
                        self.duration();
                    }
                }
                self.mas("]");
                self.mas(":");
            }
            // println!("end declaration_head");
        }
    }
    fn link(&mut self) {
        self.declaration_head();
        if self.look.1 == "{" {
            self.mas("{");
            self.notes();
            self.mas("}");
        } else {
            self.expression();
        }
    }
    fn chain(&mut self) {
        self.link();
        while self.look.1 == "->" {
            self.mas("->");
            self.link();
        }
    }
    fn chain_declaration(&mut self) {
        if self.look.0 == Id && self.peek().1 == "::" {
            self.mat(Id);
            self.mas("::");
        }
        self.chain();
    }
}
