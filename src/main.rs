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
                Ok(builder) => {
                    let functions = Functions::new(builder);
                    write(functions, 32000.0);
                }
                Err(error) => error.report(),
            },
            Err(error) => error.report(),
        }
    }
}

fn write(functions: Functions, sample_rate: f64) {
    // Find the audio end time
    // TODO: make this get done on a per-outchain basis
    let mut end = f64::MAX;
    for chain in functions.functions.values().map(|f| &f.chain) {
        let t = chain.period.end;
        if (t.lt(&end) && end == f64::MAX) || (t.gt(&end) && t != f64::MAX) {
            end = t;
        }
    }
    if end == f64::MAX {
        end = 1.0;
    }
    // println!("end: {}", end);
    // let mut last_vis_i = 0;
    // let mut vis = vec![' '; 100];
    for name in functions
        .functions
        .iter()
        .filter(|f| f.1.chain.play)
        .map(|f| f.0)
    {
        let mut song = vec![0f64; (sample_rate * end) as usize];

        for (i, mut sample) in song.iter_mut().enumerate() {
            let time = i as f64 / sample_rate;
            *sample = functions.evaluate_function(&name, &[], time, 0);

            // let vis_i = ((*sample * 50.0 + 50.0) as usize).min(99);
            // for j in last_vis_i.min(vis_i)..last_vis_i.max(vis_i) {
            //     vis[j] = '#';
            // }
            // println!("{}", vis.iter().collect::<String>());
            // for j in last_vis_i.min(vis_i)..last_vis_i.max(vis_i) {
            //     vis[j] = ' ';
            // }
            // last_vis_i = vis_i
        }
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate: sample_rate as u32,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer =
            hound::WavWriter::create(&format!("{}.wav", name.to_string()), spec).unwrap();
        let amplitude = i16::MAX as f64;
        for s in song {
            writer
                .write_sample((s * amplitude).min(amplitude) as i16)
                .unwrap();
        }
    }
}
