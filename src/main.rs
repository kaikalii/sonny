#[macro_use]
mod lexer;
mod builder;
mod parser;

use std::env;
use std::f64;

use builder::*;
use parser::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        let parser = Parser::new(&args[1]);
        let builder = parser.parse();
        write(builder, 44100.0);
    }
}

fn write(builder: Builder, sample_rate: f64) {
    // Find the audio end time
    let mut end = f64::MAX;
    for link in builder.links.values() {
        if let Time::Absolute(t) = link.borrow().period.end {
            if t.lt(&end) {
                end = t;
            }
        }
    }
    if end == f64::MAX {
        end = 60.0;
    }

    let mut song = vec![0f64; (sample_rate * end) as usize];

    for (i, mut sample) in song.iter_mut().enumerate() {
        let time = i as f64 / sample_rate;
        for chain in builder.chains.iter().filter(|c| c.play) {
            *sample = builder.evaluate_link(
                builder.links[&chain
                                  .link_names
                                  .iter()
                                  .rev()
                                  .next()
                                  .expect("Chain has no links")]
                    .borrow_mut(),
                time,
            );
        }
    }
}
