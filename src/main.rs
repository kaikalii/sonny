mod lexer;

use std::env;

use lexer::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let mut lexer = Lexer::new(&args[1]);
        loop {
            let token = lexer.lex();
            use TokenType::*;
            match token.0 {
                Done => break,
                _ => println!("{:?}", token),
            }
        }
    }
}
