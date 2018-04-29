#[macro_use]
mod lexer;
mod builder;
mod functions;
mod parser;

use std::env;
use std::f64;

use builder::*;
use functions::*;
use parser::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let parser = Parser::new(&args[1]);
        let builder = parser.parse();
        let functions = Functions::new(builder);
        write(functions, 44100.0);
    }
}

fn write(functions: Functions, sample_rate: f64) {
    // Find the audio end time
    let mut end = f64::MAX;
    for function in functions.functions.values() {
        for expression in &function.chain.links {
            if let Time::Absolute(t) = expression.period.end {
                if t.lt(&end) {
                    end = t;
                }
            }
        }
    }
    if end == f64::MAX {
        end = 60.0;
    }

    let mut song = vec![0f64; (sample_rate * end) as usize];

    for (i, mut sample) in song.iter_mut().enumerate() {
        let time = i as f64 / sample_rate;
        println!("t = {}", time);
        for name in functions
            .functions
            .iter()
            .filter(|f| f.1.chain.play)
            .map(|f| f.0)
        {
            *sample = functions.evaluate_function(&name, &[], time);
            // println!("    {:?}: {}", name, sample);
        }
    }
}
