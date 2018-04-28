use lexer::TokenType::*;
use lexer::*;

#[derive(Debug)]
pub struct Parser {
    lexer: Lexer,
    look: Token,
    next: Token,
    peeked: bool,
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
        }
    }
    pub fn parse(&mut self) {
        while self.look.0 != Done {
            self.chain_declaration();
        }
    }
    fn mat(&mut self, t: TokenType) {
        if self.look.0 == t {
            if self.peeked {
                self.peeked = false;
                self.look = self.next.clone();
            } else {
                self.look = self.lexer.lex();
            }
        } else {
            println!(
                "Unexpected {:?} : {:?} on line {}",
                self.look.0,
                self.look.1,
                self.lexer.lineno()
            );
            panic!("Bailing due to error.");
        }
    }
    fn mas(&mut self, s: &str) {
        if &self.look.1 == s {
            if self.peeked {
                self.peeked = false;
                self.look = self.next.clone();
            } else {
                self.look = self.lexer.lex();
            }
        } else {
            println!(
                "Unexpected {:?} : {:?} on line {}",
                self.look.0,
                self.look.1,
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
            if &self.look.1 == "." {
                self.mas(".");
                if self.look.0 == Num {
                    self.mat(Num);
                }
            }
        } else if &self.look.1 == "." {
            self.mas(".");
            if self.look.0 == Num {
                self.mat(Num);
            }
        } else if &self.look.1 == "pi" {
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
        println!("Notes");
    }
    fn num_backlinks(&mut self) {
        let mut broke = false;
        if self.look.0 == Num {
            self.mat(Num);
            while self.look.1 == "." {
                self.mas(".");
                if self.look.0 == Num {
                    self.mat(Num);
                } else {
                    broke = true;
                    break;
                }
            }
        }
        if broke && self.look.0 == BuiltIn {
            self.mat(BuiltIn);
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
        self.mas("$");
        self.backlinks();
        println!("Backlink");
    }
    fn term(&mut self) {
        match self.look.0 {
            Num => self.real(),
            Keyword => self.mat(Keyword),
            Id => self.mat(Id),
            Misc => if self.look.1 == "$" {
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
            println!("add");
        }
        if self.look.1 == "-" {
            self.mas("-");
            self.exp_add();
            println!("sub");
        }
    }
    fn expression(&mut self) {
        self.exp_add();
        println!("expression");
    }
    fn declaration_head(&mut self) {
        if self.look.0 == Id {
            self.mat(Id);
            self.mas(":");
            if self.look.0 == Num {
                self.duration();
                self.mas(":");
                if self.look.0 == Num {
                    self.duration();
                    self.mas(":");
                }
            }
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
        println!("Link");
    }
    fn chain(&mut self) {
        self.link();
        if self.look.1 == "->" {
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
