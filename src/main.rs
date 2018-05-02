extern crate colored;
extern crate either;
extern crate hound;

#[macro_use]
mod lexer;
mod builder;
mod error;
mod functions;
mod parser;

use std::env;
use std::{f64, i16};

use builder::*;
use functions::*;
use parser::*;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() >= 2 {
        match Parser::new(&args[1]) {
            Ok(parser) => match parser.parse() {
                Ok(mut builder) => {
                    builder.make_functions();
                    write(builder, 4000.0);
                }
                Err(error) => error.report(),
            },
            Err(error) => error.report(),
        }
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
        let mut song = vec![0f64; (sample_rate * end) as usize];

        for (i, mut sample) in song.iter_mut().enumerate() {
            let time = i as f64 / sample_rate;
            *sample = builder.evaluate_function(&name, &[], time, 0);
        }
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
                    format!("{}", name)
                }
            ),
            spec,
        ).unwrap();
        let amplitude = i16::MAX as f64;
        for s in song {
            writer
                .write_sample((s * amplitude).min(amplitude) as i16)
                .unwrap();
        }
    }
}
