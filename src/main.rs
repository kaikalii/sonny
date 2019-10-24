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

use std::{
    collections::VecDeque,
    env, f64, i16,
    io::{stdout, Write},
    time::Instant,
};

use colored::*;

use builder::*;
use error::*;
use parser::*;

fn main() {
    let mut args = env::args();
    args.next();
    let mut sample_rate = 32000.0;
    let mut window_size = 4000;
    let mut buffer_size = 10;
    let mut file_name = None;
    let mut start_time = 0f64;
    let mut end_time = None;
    let mut play = false;
    // Parse command args for input and flags
    while let Some(ref arg) = args.next() {
        match arg.to_string().as_ref() {
            "-r" | "--sample_rate" => {
                if let Some(ref sr_str) = args.next() {
                    if let Ok(sr) = sr_str.parse() {
                        sample_rate = sr;
                    } else {
                        println!("Invalid sample rate.");
                        return;
                    }
                }
            }
            "-w" | "--window" => {
                if let Some(ref w_str) = args.next() {
                    if let Ok(w) = w_str.parse() {
                        window_size = w;
                    } else {
                        println!("Invalid window size.");
                        return;
                    }
                }
            }
            "-b" | "--buffer" => {
                if let Some(ref b_str) = args.next() {
                    if let Ok(b) = b_str.parse() {
                        buffer_size = b;
                    } else {
                        println!("Invalid buffer size.");
                        return;
                    }
                }
            }
            "-s" | "--start" => {
                if let Some(ref s_str) = args.next() {
                    if let Ok(s) = s_str.parse() {
                        start_time = s;
                    } else {
                        println!("Invalid start time.");
                        return;
                    }
                }
            }
            "-e" | "--end" => {
                if let Some(ref e_str) = args.next() {
                    if let Ok(e) = e_str.parse() {
                        end_time = Some(e);
                    } else {
                        println!("Invalid end_time.");
                        return;
                    }
                }
            }
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
                            (default is 4000)
    -b | --buffer           Set the size of the buffer before the
                            window (default is 10)
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
                Ok(builder) => {
                    // output sound
                    if let Err(error) = write(
                        &builder,
                        sample_rate,
                        window_size,
                        buffer_size,
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
    builder: &Builder,
    sample_rate: f64,
    window_size: usize,
    buffer_size: usize,
    start_time: f64,
    end_time: Option<f64>,
    play: bool,
) -> SonnyResult<()> {
    // Find the audio end time
    // TODO: make this only take into account chains that are actually used
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

    // output the main chain
    if let Some(name) = builder.chains.iter().find(|f| f.1.play).map(|f| f.0) {
        // populate the sample array with its own indicies because par_iter doesn't have enumerate()
        let mut song = vec![0f64; (sample_rate * (end - start_time)) as usize];
        // run each sample window as a batch
        let mut then = Instant::now(); // Keeps track of the time when the last window iteration started
        let mut last_elapsed = VecDeque::new(); // Keeps a moving list of elapsed time values for a running average
        let start_instant = Instant::now(); // The time the evaluation started

        // Main generation loop
        let window_count = (song.len() as f64 / window_size as f64).ceil() as usize;
        for window_start in (0..window_count).map(|x| x * window_size) {
            // Determine the buffer size and adjusted window start
            let this_buffer_size = if window_start == 0 { 0 } else { buffer_size };
            let window_start = window_start - this_buffer_size;
            // Determine the time
            let time = window_start as f64 / sample_rate + start_time;

            // Print a progress bar
            let progress = (time / end * 41.0) as usize;
            print!(
                "\r{} [{}>{}] ",
                format!("{:.2} / {:.2} s", time, end - start_time).cyan(),
                (0..progress).map(|_| '=').collect::<String>(),
                (0..(40 - progress)).map(|_| ' ').collect::<String>()
            );
            // Print an eta
            let now = Instant::now();
            let elapsed = now.duration_since(then);
            then = now;
            let elapsed = elapsed.as_secs() as f64 + f64::from(elapsed.subsec_nanos()) / 1e9;
            last_elapsed.push_back(elapsed);
            if last_elapsed.len() > 30 {
                last_elapsed.pop_front();
            }
            let rate = ((window_size + this_buffer_size) as f64 / sample_rate)
                / (last_elapsed.iter().sum::<f64>() / last_elapsed.len() as f64);
            let eta = (end - time) / rate;
            print!("eta: {}", format!("{:.2}s", eta).cyan());
            stdout().flush().expect("Unable to flush stdout");

            // Evaluate
            if time >= end {
                break;
            }
            let window_result = builder.evaluate_chain(
                &name,
                &[],
                time,
                window_size.min(song.len() - window_start),
                this_buffer_size,
                sample_rate,
            )?;
            for (i, r) in window_result.into_iter().skip(this_buffer_size).enumerate() {
                song[i + window_start] = f64::from(r);
            }
        }
        // Print the final progress bar
        let total_elapsed = Instant::now().duration_since(start_instant);
        print!("\r                                                                                               \r");
        println!(
            "{} [{}>] elapsed: {}",
            format!("{:.2} / {:.2} s", end - start_time, end - start_time).cyan(),
            (0..40).map(|_| '=').collect::<String>(),
            format!(
                "{:5.2} s",
                total_elapsed.as_secs() as f64 + f64::from(total_elapsed.subsec_nanos()) / 1e9
            )
            .cyan()
        );

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
            let amplitude = f64::from(i16::MAX);
            for s in song {
                writer
                    .write_sample((s * amplitude).min(amplitude) as i16)
                    .unwrap();
            }
        }

        if play && open::that(&filename).is_err() {
            return Err(Error::new(ErrorSpec::CantOpenOutputFile));
        }
    }
    Ok(())
}
