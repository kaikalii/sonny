extern crate colored;
extern crate either;
extern crate hound;
extern crate rayon;

#[macro_use]
mod lexer;
mod builder;
mod error;
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
    let mut window_size = 100;
    let mut file_name = None;
    // Parse command args for input and flags
    while let Some(ref arg) = args.next() {
        match arg.to_string().as_ref() {
            "-r" | "--sample_rate" => if let Some(ref sr_str) = args.next() {
                if let Ok(sr) = sr_str.parse() {
                    sample_rate = sr;
                } else {
                    println!("Invalid sample rate.");
                    return;
                }
            },
            "-w" | "--window" => if let Some(ref w_str) = args.next() {
                if let Ok(w) = w_str.parse() {
                    window_size = w;
                } else {
                    println!("Invalid window size.");
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
    -r | --sample_rate      Set the sample rate of the output file
                            in samples/second (default is 32000)
	-w | --window			Set the size of the processing window
"
                );
                return;
            }
            _ => file_name = Some(arg.to_string()),
        }
    }
    if let Some(ref file_name) = file_name {
        // Initialize the parser
        match Parser::new(file_name, Builder::new()) {
            // start parsing the file
            Ok(parser) => match parser.parse(false) {
                // make functions
                Ok(mut builder) => {
                    // output sound
                    write(builder, sample_rate, window_size);
                }
                Err(error) => error.report(),
            },
            Err(error) => error.report(),
        }
    } else {
        println!("Usage: \n    sonny <filname> [options]\n    Type \"sonny -h\" or \"sonny --help\" for usage details.");
    }
}

fn write(builder: Builder, sample_rate: f64, window_size: usize) {
    // Find the audio end time
    // TODO: make this get done on a per-outchain basis
    let mut end: f64 = 1.0;
    for chain in builder.chains.values() {
        if let ChainLinks::OnlyNotes(ref _notes_or_ids, period) = chain.links {
            end = end.max(period.end);
        }
    }
    end = end.max(builder.end_time);

    // output each outchain
    for name in builder.chains.iter().filter(|f| f.1.play).map(|f| f.0) {
        // populate the sample array with its own indicies so because par_iter doesn't have enumerate()
        let mut song = vec![(0f64, 0usize); (sample_rate * end) as usize];
        for (i, (_, ref mut x)) in song.iter_mut().enumerate() {
            *x = i;
        }
        // run each sample window as a batch
        for window_start in (0..(song.len() / window_size)).map(|x| x * window_size) {
            song[window_start..(window_start + window_size)]
                .par_iter_mut()
                .for_each(|(sample, i)| {
                    let time = *i as f64 / sample_rate;
                    *sample = builder.evaluate_function(&name, &[], time).to_f64();
                });
        }

        // Write the audio file
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: sample_rate as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(
            &format!(
                "{}.wav",
                if let ChainName::Scoped(chain_name) = name {
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
