mod lexer;

use std::env;
use std::fs::File;

use lexer::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let mut file = File::open(&args[1]).expect(&format!("Unable to open file: {}", args[1]));
        loop {
            let token = lexer(&mut file);
            use TokenType::*;
            match token.t {
                Done => break,
                _ => println!("{:?}", token),
            }
        }
    }
}
