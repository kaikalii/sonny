#[macro_use]
mod lexer;
mod parser;

use std::env;

use parser::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let mut parser = Parser::new(&args[1]);
        parser.parse();
    }
}
