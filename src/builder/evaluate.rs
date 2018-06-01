// This Module contains functions for evaluating chains

use std::f64;

use rayon::prelude::*;

use rustfft::{num_complex::Complex, num_traits::Zero, FFTplanner};

use builder::{variable::*, *};

type Variables = Vec<Variable>;

impl ChainLinks {
    // When called on OnlyNotes links, this function returns the note whos period contains
    // the given time. When initially called, the start offset should usually be 0.0, but
    // recursive calls increment it as more links are searched through.
    pub fn find_note(&self, time: f64, start_offset: f64, builder: &Builder) -> Option<Note> {
        if let ChainLinks::OnlyNotes(ref notes_or_ids, period) = *self {
            // If this period of these links does not contain the note, then immediately return.
            if period.start + start_offset > time || period.end + start_offset <= time {
                None
            } else {
                let mut local_offset = start_offset;
                // For each note or id
                for notes_or_id in notes_or_ids {
                    match notes_or_id {
                        // If the link is an Id
                        NotesOrId::Id(ref id) => {
                            // Find the associated chain
                            if let Some(chain) = builder.find_chain(id) {
                                // Make sure the chain has OnlyNote links
                                if let ChainLinks::OnlyNotes(.., period) = chain.links {
                                    let this_offset = local_offset;
                                    // Increase the local offset by the links' period
                                    local_offset += period.duration();
                                    // Recursively call this function
                                    if let Some(note) =
                                        chain.links.find_note(time, this_offset, builder)
                                    {
                                        return Some(note);
                                    }
                                } else {
                                    panic!(
                                        "somehow, an OnlyNotes chain had the id of a generic chain"
                                    );
                                }
                            } else {
                                panic!("Unable to find '{}'", id);
                            }
                        }
                        // If the link is Notes
                        NotesOrId::Notes(ref notes) => {
                            // Search the notes for one whose period contains the time.
                            for note in notes {
                                if note.period.start + local_offset <= time
                                    && note.period.end + local_offset > time
                                {
                                    return Some(Note {
                                        pitches: note.pitches.clone(),
                                        period: Period {
                                            start: note.period.start + local_offset,
                                            end: note.period.end + local_offset,
                                        },
                                    });
                                }
                            }
                            panic!("Unable to find note in notes even though it should have been there.");
                        }
                    }
                }
                panic!(
                    "Unable to find note in notes or ids even though it should have been there."
                );
            }
        } else {
            panic!("Can find notes in generic chain");
        }
    }
}

impl Builder {
    // Evalutates an oeprand with the given arguments and depth
    fn evaluate_operand(
        &self,
        operand: &Operand,
        name: &ChainName,
        args: &[&Variables],
        time: f64,
        window_size: usize,
        buffer_size: usize,
        sample_rate: f64,
    ) -> Variables {
        use Operand::*;
        match *operand {
            // for Nums, simply return the num
            Var(ref x) => vec![x.clone(); buffer_size + window_size],
            // for Ids, call the associated function
            Id(ref id) => {
                self.evaluate_chain(id, args, time, window_size, buffer_size, sample_rate)
            }
            // for Notes Properties...
            Properties(ref id) => if let Some(chain) = self.find_chain(id) {
                (0..(buffer_size + window_size))
                    .collect::<Vec<usize>>()
                    .into_par_iter()
                    .map(|i| time + i as f64 / sample_rate)
                    .map(|t| {
                        // Ensure that this is in fact an OnlyNotes chain.
                        // This check should always succeed because the parser
                        // checks it during the building phase
                        if let ChainLinks::OnlyNotes(..) = chain.links {
                            // Try to find the note and return it if it is found
                            if let Some(note) = chain.links.find_note(t, 0.0, &self) {
                                Variable::Array(vec![
                                    Variable::Array(
                                        note.pitches.iter().map(|p| Variable::Number(*p)).collect(),
                                    ),
                                    Variable::Number(note.period.start),
                                    Variable::Number(note.period.end),
                                    Variable::Number(note.period.duration()),
                                ])
                            // return zero if time is after the period of the notes
                            } else {
                                Variable::Number(0.0)
                            }
                        } else {
                            panic!("Reference chain is not a note chain");
                        }
                    })
                    .collect()
            } else {
                panic!("Unknown id {:?}", id)
            },
            // For time, simply return the time
            Time => (0..(buffer_size + window_size))
                .collect::<Vec<usize>>()
                .into_par_iter()
                .map(|i| Variable::Number(time + i as f64 / sample_rate))
                .collect(),
            // For window size, simply return the window size
            WindowSize => vec![Variable::Number(window_size as f64); buffer_size + window_size],
            // For buffer size, simply return the buffer size
            BufferSize => vec![Variable::Number(buffer_size as f64); buffer_size + window_size],
            // For sample rate, simply return the sample rate
            SampleRate => vec![Variable::Number(sample_rate as f64); buffer_size + window_size],
            // For window index, simpe return a sequential array up to the window size
            WindowIndex => (0..(buffer_size + window_size))
                .map(|x| Variable::Number(f64::from(x as u32)))
                .collect(),
            // For Backlinks, reference the arguments passed
            BackLink(num) => args[num - 1].clone(),
            // It's technically not possible to have notes here, since
            // all notes operands are removed when a chain is finalized.
            // Just make sure. You never know. This might change.
            Notes(ref notes) => (0..(buffer_size + window_size))
                .collect::<Vec<usize>>()
                .into_par_iter()
                .map(|i| time + i as f64 / sample_rate)
                .map(|t| {
                    let mut result = Vec::new();
                    for note in notes {
                        if note.period.contains(t) {
                            result = note.pitches.clone();
                            break;
                        }
                    }
                    Variable::Array(result.into_iter().map(|p| Variable::Number(p)).collect())
                })
                .collect(),
            // Evaluate expressions
            Expression(ref expression) => self.evaluate_expression(
                expression,
                name,
                args,
                time,
                window_size,
                buffer_size,
                sample_rate,
            ),
            // Evaluate an array
            Array(ref expressions) => {
                let uncollated: Vec<Variables> = expressions
                    .iter()
                    .map(|expression| {
                        self.evaluate_expression(
                            expression,
                            name,
                            args,
                            time,
                            window_size,
                            buffer_size,
                            sample_rate,
                        )
                    })
                    .collect();
                let mut result =
                    vec![
                        Variable::Array(vec![Variable::Number(0.0); expressions.len()]);
                        buffer_size + window_size
                    ];
                for (i, vars) in uncollated.into_iter().enumerate() {
                    for (j, var) in vars.into_iter().enumerate() {
                        if let Variable::Array(ref mut v) = result[j] {
                            v[i] = var;
                        }
                    }
                }
                result
            }
        }
    }

    // Evaluate an expression with the given arguments and time
    fn evaluate_expression(
        &self,
        expression: &Expression,
        name: &ChainName,
        args: &[&Variables],
        time: f64,
        window_size: usize,
        buffer_size: usize,
        sample_rate: f64,
    ) -> Variables {
        // Evaluate Operands
        use self::Operation::*;
        let ops = expression.0.operands();
        let x = self.evaluate_operand(
            ops.0,
            name,
            args,
            time,
            window_size,
            buffer_size,
            sample_rate,
        );
        let y = ops.1.map(|op| {
            self.evaluate_operand(op, name, args, time, window_size, buffer_size, sample_rate)
        });
        let z = ops.2.map(|op| {
            self.evaluate_operand(op, name, args, time, window_size, buffer_size, sample_rate)
        });

        // Evaluate Operation
        match expression.0 {
            Add(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in add").into_par_iter())
                .map(|(x, y)| x + y)
                .collect(),
            Subtract(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in sub").into_par_iter())
                .map(|(x, y)| x - y)
                .collect(),
            Multiply(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in mul").into_par_iter())
                .map(|(x, y)| x * y)
                .collect(),
            Divide(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in div").into_par_iter())
                .map(|(x, y)| x / y)
                .collect(),
            Remainder(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in rem").into_par_iter())
                .map(|(x, y)| x % y)
                .collect(),
            Power(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in pow").into_par_iter())
                .map(|(x, y)| x.pow(y))
                .collect(),
            Min(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in min").into_par_iter())
                .map(|(x, y)| x.min(y))
                .collect(),
            Max(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in max").into_par_iter())
                .map(|(x, y)| x.max(y))
                .collect(),
            LessThan(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in less than").into_par_iter())
                .map(|(x, y)| {
                    if x < y {
                        Variable::Number(1.0)
                    } else {
                        Variable::Number(0.0)
                    }
                })
                .collect(),
            GreaterThan(..) => x.into_par_iter()
                .zip(
                    y.expect("failed to unwrap y in greater than")
                        .into_par_iter(),
                )
                .map(|(x, y)| {
                    if x > y {
                        Variable::Number(1.0)
                    } else {
                        Variable::Number(0.0)
                    }
                })
                .collect(),
            LessThanOrEqual(..) => x.into_par_iter()
                .zip(
                    y.expect("failed to unwrap y in less than or equal")
                        .into_par_iter(),
                )
                .map(|(x, y)| {
                    if x <= y {
                        Variable::Number(1.0)
                    } else {
                        Variable::Number(0.0)
                    }
                })
                .collect(),
            GreaterThanOrEqual(..) => x.into_par_iter()
                .zip(
                    y.expect("failed to unwrap y in greater than or equal")
                        .into_par_iter(),
                )
                .map(|(x, y)| {
                    if x >= y {
                        Variable::Number(1.0)
                    } else {
                        Variable::Number(0.0)
                    }
                })
                .collect(),
            Equal(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in equal").into_par_iter())
                .map(|(x, y)| {
                    if x == y {
                        Variable::Number(1.0)
                    } else {
                        Variable::Number(0.0)
                    }
                })
                .collect(),
            NotEqual(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in not equal").into_par_iter())
                .map(|(x, y)| {
                    if x != y {
                        Variable::Number(1.0)
                    } else {
                        Variable::Number(0.0)
                    }
                })
                .collect(),
            And(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in and").into_par_iter())
                .map(|(x, y)| x.min(y))
                .collect(),
            Or(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in or").into_par_iter())
                .map(|(x, y)| x.max(y))
                .collect(),
            Negate(..) => x.into_par_iter().map(|x| -x).collect(),
            Sine(..) => x.into_par_iter().map(|x| x.sin()).collect(),
            Cosine(..) => x.into_par_iter().map(|x| x.cos()).collect(),
            Ceiling(..) => x.into_par_iter().map(|x| x.ceil()).collect(),
            Floor(..) => x.into_par_iter().map(|x| x.floor()).collect(),
            AbsoluteValue(..) => x.into_par_iter().map(|x| x.abs()).collect(),
            Logarithm(..) => x.into_par_iter().map(|x| x.ln()).collect(),
            Operand(..) => x,
            Ternary(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in ternay").into_par_iter())
                .zip(z.expect("failed to unwrap z in ternay").into_par_iter())
                .map(|((x, y), z)| if x != Variable::Number(0.0) { y } else { z })
                .collect(),
            Index(..) => x.into_par_iter()
                .zip(y.expect("failed to unwrap y in index").into_par_iter())
                .map(|(x, y)| x[y].clone())
                .collect(),
            SubArray(..) => x.into_par_iter()
                .zip(
                    y.expect("failed to unwrap y in sub_array")
                        .into_par_iter()
                        .zip(z.expect("failed to unwrap z in sub_array").into_par_iter()),
                )
                .map(|(x, (y, z))| x.sub_array(y, z))
                .collect(),
            Average(..) => x.into_par_iter().map(|x| x.average()).collect(),
            FFT(..) => {
                let mut input: Vec<Complex<f64>> = x.iter()
                    .map(|sample| {
                        if let Variable::Number(s) = *sample {
                            Complex::new(s, 0.0)
                        } else {
                            panic!("tried to run fft on array")
                        }
                    })
                    .collect();
                let mut output = vec![Complex::zero(); x.len()];
                let mut planner = FFTplanner::new(false);
                let fft = planner.plan_fft(x.len());
                fft.process(&mut input, &mut output);
                let fft_result = Variable::Array(vec![
                    Variable::Array(
                        (0..((buffer_size + window_size) / 2))
                            .map(|i| {
                                Variable::Number(
                                    i as f64 * sample_rate / (buffer_size + window_size) as f64,
                                )
                            })
                            .collect(),
                    ),
                    Variable::Array({
                        let mut bins: Vec<Variable> = output
                            .into_iter()
                            .take((buffer_size + window_size) / 2)
                            .map(|x| Variable::Number((x.re.powf(2.0) + x.im.powf(2.0)).powf(0.5)))
                            .collect();
                        let max = bins.iter()
                            .max_by(|a, b| a.partial_cmp(b).expect("NaN in fft"))
                            .expect("empty fft bins")
                            .clone();
                        bins.into_iter().map(|x| x / max.clone()).collect()
                    }),
                ]);
                vec![fft_result.clone(); buffer_size + window_size]
            }
            Window(..) => vec![Variable::Array(x.clone()); buffer_size + window_size],
        }
    }

    // Evaluate a chain
    pub fn evaluate_chain(
        &self,
        name: &ChainName,
        args: &[&Variables],
        time: f64,
        window_size: usize,
        buffer_size: usize,
        sample_rate: f64,
    ) -> Variables {
        if let Some(chain) = self.find_chain(name) {
            match chain.links {
                ChainLinks::Generic(ref expressions) => {
                    let mut results: Vec<Variables> = Vec::new();
                    for (_i, expression) in expressions.iter().enumerate() {
                        let mut results_collector: Vec<Variables> = Vec::new();
                        {
                            // Create the args to be passed to the evaluate_expression() call
                            let mut these_args: Vec<&Variables> = Vec::new();
                            // Add all previous arg results of this chain reversed
                            these_args.extend(results.iter().rev());
                            // Add the args coming into this chain
                            these_args.extend(args);

                            results_collector.push(self.evaluate_expression(
                                expression,
                                name,
                                &these_args,
                                time,
                                window_size,
                                buffer_size,
                                sample_rate,
                            ));
                        }
                        results.extend(results_collector.into_iter());
                    }
                    results
                        .into_iter()
                        .last()
                        .expect("generic chain gave no last result")
                }
                ChainLinks::OnlyNotes(..) => (0..(buffer_size + window_size))
                    .collect::<Vec<usize>>()
                    .into_par_iter()
                    .map(|i| time + i as f64 / sample_rate)
                    .map(|t| {
                        Variable::Array(
                            chain
                                .links
                                .find_note(t, 0.0, &self)
                                .map(|n| {
                                    n.pitches.into_iter().map(|p| Variable::Number(p)).collect()
                                })
                                .unwrap_or_else(Vec::new),
                        )
                    })
                    .collect(),
            }
        } else {
            panic!("No function named '{}'", name);
        }
    }
}
