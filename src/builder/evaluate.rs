// This Module contains functions for evaluating chains

use std::f64;

use builder::{variable::*, *};

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
                                        pitch: note.pitch,
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
        args: &[Variable],
        time: f64,
    ) -> Variable {
        use Operand::*;
        match *operand {
            // for Nums, simply return the num
            Var(ref x) => x.clone(),
            // for Ids, call the associated function
            Id(ref id) => self.evaluate_function(id, args, time),
            // for Notes Properties...
            Property(ref id, property) => if let Some(chain) = self.find_chain(id) {
                // Ensure that this is in fact an OnlyNotes chain.
                // This check should always succeed because the parser
                // checks it during the building phase
                if let ChainLinks::OnlyNotes(..) = chain.links {
                    // Try to find the note and return it if it is found
                    if let Some(note) = chain.links.find_note(time, 0.0, &self) {
                        use builder::Property::*;
                        match property {
                            Start => Variable::Number(note.period.start),
                            End => Variable::Number(note.period.end),
                            Duration => Variable::Number(note.period.duration()),
                        }
                    // return zero if time is after the period of the notes
                    } else {
                        Variable::Number(0.0)
                    }
                } else {
                    panic!("Reference chain is not a note chain");
                }
            } else {
                panic!("Unknown id {:?}", id)
            },
            // For time, simply return the time
            Time => Variable::Number(time),
            // For Backlinks, reference the arguments passed
            BackLink(num) => args[num - 1].clone(),
            // It's technically not possible to have notes here, since
            // all notes operands are removed when a chain is finalized.
            // Just make sure. You never know.
            Notes(ref notes) => {
                let mut result = 0.0;
                for note in notes {
                    if note.period.contains(time) {
                        result = note.pitch;
                        break;
                    }
                }
                Variable::Number(result)
            }
            // Evaluate expressions
            Expression(ref expression) => self.evaluate_expression(expression, name, args, time),
        }
    }

    // Evaluate an expression with the given arguments and time
    fn evaluate_expression(
        &self,
        expression: &Expression,
        name: &ChainName,
        args: &[Variable],
        time: f64,
    ) -> Variable {
        use self::Operation::*;
        let (a, b, c) = expression.0.operands();
        let x = self.evaluate_operand(a, name, args, time);
        let y = b.map(|b| self.evaluate_operand(b, name, args, time));
        let z = c.map(|c| self.evaluate_operand(c, name, args, time));
        match expression.0 {
            Add(..) => x + y.expect("failed to unwrap y in add"),
            Subtract(..) => x - y.expect("failed to unwrap y in subtract"),
            Multiply(..) => x * y.expect("failed to unwrap y in multiply"),
            Divide(..) => x / y.expect("failed to unwrap y in divide"),
            Remainder(..) => x % y.expect("failed to unwrap y in remainder"),
            Power(..) => x.pow(y.expect("failed to unwrap y in min")),
            Min(..) => x.min(y.expect("failed to unwrap y in min")),
            Max(..) => x.max(y.expect("failed to unwrap y in max")),
            LessThan(..) => if x < y.expect("failed to unwrap y in compare") {
                Variable::Number(1.0)
            } else {
                Variable::Number(0.0)
            },
            GreaterThan(..) => if x > y.expect("failed to unwrap y in compare") {
                Variable::Number(1.0)
            } else {
                Variable::Number(0.0)
            },
            LessThanOrEqual(..) => if x <= y.expect("failed to unwrap y in compare") {
                Variable::Number(1.0)
            } else {
                Variable::Number(0.0)
            },
            GreaterThanOrEqual(..) => if x >= y.expect("failed to unwrap y in compare") {
                Variable::Number(1.0)
            } else {
                Variable::Number(0.0)
            },
            Equal(..) => if x == y.expect("failed to unwrap y in equal") {
                Variable::Number(1.0)
            } else {
                Variable::Number(0.0)
            },
            NotEqual(..) => if x != y.expect("failed to unwrap y in not equal") {
                Variable::Number(1.0)
            } else {
                Variable::Number(0.0)
            },
            Or(..) => x.max(y.expect("failed to unwrap y in or")),
            And(..) => x.min(y.expect("failed to unwrap y in or")),
            Negate(..) => -x,
            Sine(..) => x.sin(),
            Cosine(..) => x.cos(),
            Ceiling(..) => x.ceil(),
            Floor(..) => x.floor(),
            AbsoluteValue(..) => x.abs(),
            Logarithm(..) => x.ln(),
            Operand(..) => x,
            Ternary(..) => if x != Variable::Number(0.0) {
                y.expect("failed to unwrap y in ternay")
            } else {
                z.expect("failed to unwrap z in ternay")
            },
        }
    }

    // Pretend a chain is a function and evaluate it as such
    pub fn evaluate_function(&self, name: &ChainName, args: &[Variable], time: f64) -> Variable {
        if let Some(chain) = self.find_chain(name) {
            match chain.links {
                ChainLinks::Generic(ref expressions) => {
                    let mut results: Vec<Variable> = Vec::new();
                    for (_i, expression) in expressions.iter().enumerate() {
                        let mut these_args: Vec<Variable> = Vec::new();
                        for r in results.iter().rev() {
                            these_args.push(r.clone());
                        }
                        for a in args {
                            these_args.push(a.clone());
                        }
                        results.push(self.evaluate_expression(expression, name, &these_args, time));
                    }
                    results
                        .into_iter()
                        .last()
                        .expect("generic chain gave no last result")
                }
                ChainLinks::OnlyNotes(..) => Variable::Number(
                    chain
                        .links
                        .find_note(time, 0.0, &self)
                        .map(|n| n.pitch)
                        .unwrap_or(0.0),
                ),
            }
        } else {
            panic!("No function named '{}'", name);
        }
    }
}
