extern crate colored;
extern crate either;
extern crate find_folder;
extern crate hound;
extern crate open;
extern crate rayon;
extern crate rustfft;

mod builder;
mod error;
mod lexer;
mod parser;

use std::{env, f64, i16};

use builder::*;
use error::*;
use parser::*;

fn main() {
    let mut args = env::args();
    args.next();
    let mut sample_rate = 32000.0;
    let mut window_size = 4000;
    let mut file_name = None;
    let mut start_time = 0f64;
    let mut end_time = None;
    let mut play = false;
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
            "-s" | "--start" => if let Some(ref s_str) = args.next() {
                if let Ok(s) = s_str.parse() {
                    start_time = s;
                } else {
                    println!("Invalid start time.");
                    return;
                }
            },
            "-e" | "--end" => if let Some(ref e_str) = args.next() {
                if let Ok(e) = e_str.parse() {
                    end_time = Some(e);
                } else {
                    println!("Invalid end_time.");
                    return;
                }
            },
            "-p" | "--play" => play = true,
            "-h" | "--help" => {
                println!(
                    "\n\
Usage:
    sonny <filename> [options]

Options:
    -h | --help             Display this message
    -r | --sample_rate      Set the sample rate of the output file
                            in samples/second (default is 32000)
    -w | --window           Set the size of the processing window
                            (default it )
    -s | --start            Set the start time of the output file
    -e | --end              Set the end time of the output file
    -p | --play             Plays the output file after it is
                            finished generating.
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
                    if let Err(error) = write(
                        builder,
                        sample_rate,
                        window_size,
                        start_time,
                        end_time,
                        play,
                    ) {
                        error.report();
                    }
                }
                Err(error) => error.report(),
            },
            Err(error) => error.report(),
        }
    } else {
        println!("Usage: \n    sonny <filname> [options]\n    Type \"sonny -h\" or \"sonny --help\" for usage details.");
    }
}

fn write(
    builder: Builder,
    sample_rate: f64,
    window_size: usize,
    start_time: f64,
    end_time: Option<f64>,
    play: bool,
) -> SonnyResult<()> {
    // Find the audio end time
    // TODO: make this get done on a per-outchain basis
    let mut end: f64 = 1.0;
    for chain in builder.chains.values() {
        if let ChainLinks::OnlyNotes(ref _notes_or_ids, period) = chain.links {
            end = end.max(period.end);
        }
    }
    end = end.max(builder.end_time);
    if let Some(end_time) = end_time {
        end = end_time;
    }

    // output each outchain
    if let Some(name) = builder.chains.iter().find(|f| f.1.play).map(|f| f.0) {
        // populate the sample array with its own indicies so because par_iter doesn't have enumerate()
        let mut song = vec![0f64; (sample_rate * (end - start_time)) as usize];
        // run each sample window as a batch
        let mut time;
        for window_start in (0..(song.len() / window_size)).map(|x| x * window_size) {
            time = window_start as f64 / sample_rate + start_time;
            if time >= end {
                break;
            }
            let window_result = builder.evaluate_chain(&name, &[], time, window_size, sample_rate);
            for (i, r) in window_result.into_iter().enumerate() {
                song[i + window_start] = r.to_f64();
            }
        }

        // Write the audio file
        let filename;
        {
            let spec = hound::WavSpec {
                channels: 1,
                sample_rate: sample_rate as u32,
                bits_per_sample: 16,
                sample_format: hound::SampleFormat::Int,
            };
            filename = format!(
                "{}.wav",
                if let ChainName::Scoped(chain_name) = name {
                    chain_name.split("::").last().unwrap().to_string()
                } else {
                    name.to_string()
                }
            );
            let mut writer = hound::WavWriter::create(&filename, spec).unwrap();
            let amplitude = i16::MAX as f64;
            for s in song {
                writer
                    .write_sample((s * amplitude).min(amplitude) as i16)
                    .unwrap();
            }
        }

        if play {
            if open::that(&filename).is_err() {
                return Err(Error::new(ErrorSpec::CantOpenOutputFile));
            }
        }
    }
    Ok(())
}
