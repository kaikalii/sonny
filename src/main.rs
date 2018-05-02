extern crate colored;
extern crate either;
extern crate hound;
extern crate rayon;

#[macro_use]
mod lexer;
mod builder;
mod error;
mod functions;
mod parser;

use std::env;
use std::{f64, i16};

use rayon::prelude::*;

use builder::*;
use parser::*;

fn main() {
    let mut args = env::args();
    args.next();
    let mut sample_rate = 32000.0;
    let mut file_name = None;
    while let Some(ref arg) = args.next() {
        match arg.to_string().as_ref() {
            "-s" | "--sample_rate" => if let Some(ref sr_str) = args.next() {
                if let Ok(sr) = sr_str.parse() {
                    sample_rate = sr;
                } else {
                    println!("Invalid sample rate.");
                    return;
                }
            },
            "-h" | "--help" => {
                println!(
                    "\n\
Usage:
    sonny <filename> [options]

Options:
    -h | --help             Display this message
    -s | --sample_rate      Set the sample rate of the output file
                            in samples/second (default is 32000)
"
                );
                return;
            }
            _ => file_name = Some(arg.to_string()),
        }
    }
    if let Some(ref file_name) = file_name {
        match Parser::new(file_name) {
            Ok(parser) => match parser.parse() {
                Ok(mut builder) => {
                    builder.make_functions();
                    write(builder, sample_rate);
                }
                Err(error) => error.report(),
            },
            Err(error) => error.report(),
        }
    } else {
        println!("Usage: \n    sonny <filname> [options]\n    Type \"sonny -h\" or \"sonny --help\" for usage details.");
    }
}

fn write(builder: Builder, sample_rate: f64) {
    // Find the audio end time
    // TODO: make this get done on a per-outchain basis
    let mut end: f64 = 1.0;
    for chain in builder.chains.values() {
        if let ChainLinks::OnlyNotes(ref _notes_or_ids, period) = chain.links {
            end = end.max(period.end);
        }
    }
    end = end.max(builder.end_time);
    for name in builder.chains.iter().filter(|f| f.1.play).map(|f| f.0) {
        let mut song = vec![(0f64, 0usize); (sample_rate * end) as usize];
        for (i, (_, ref mut x)) in song.iter_mut().enumerate() {
            *x = i;
        }

        song.par_iter_mut().for_each(|(sample, i)| {
            let time = *i as f64 / sample_rate;
            *sample = builder.evaluate_function(&name, &[], time, 0);
        });
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: sample_rate as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(
            &format!(
                "{}.wav",
                if let ChainName::String(chain_name) = name {
                    chain_name.split("::").last().unwrap().to_string()
                } else {
                    name.to_string()
                }
            ),
            spec,
        ).unwrap();
        let amplitude = i16::MAX as f64;
        for (s, _) in song {
            writer
                .write_sample((s * amplitude).min(amplitude) as i16)
                .unwrap();
        }
    }
}
