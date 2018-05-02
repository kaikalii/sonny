use std::collections::{HashMap, HashSet};
use std::f64;
use std::fmt;

use error::{ErrorSpec::*, *};
use lexer::CodeLocation;

#[derive(Debug, Clone, Copy)]
pub enum Property {
    Start,
    End,
    Duration,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Num(f64),
    Id(ChainName),
    Property(ChainName, Property),
    BackLink(usize),
    Time,
    Notes(Vec<Note>),
    Expression(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Operation {
    Add(Operand, Operand),
    Subtract(Operand, Operand),
    Multiply(Operand, Operand),
    Divide(Operand, Operand),
    Remainder(Operand, Operand),
    Power(Operand, Operand),
    Min(Operand, Operand),
    Max(Operand, Operand),
    Negate(Operand),
    Sine(Operand),
    Cosine(Operand),
    Floor(Operand),
    Ceiling(Operand),
    AbsoluteValue(Operand),
    Logarithm(Operand),
    Operand(Operand),
}

impl Operation {
    pub fn operands(&self) -> (&Operand, Option<&Operand>) {
        use self::Operation::*;
        match *self {
            Add(ref a, ref b)
            | Subtract(ref a, ref b)
            | Multiply(ref a, ref b)
            | Divide(ref a, ref b)
            | Remainder(ref a, ref b)
            | Power(ref a, ref b)
            | Min(ref a, ref b)
            | Max(ref a, ref b) => (a, Some(b)),
            Negate(ref a) | Sine(ref a) | Cosine(ref a) | Ceiling(ref a) | Floor(ref a)
            | AbsoluteValue(ref a) | Logarithm(ref a) | Operand(ref a) => (a, None),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Period {
    pub start: f64,
    pub end: f64,
}

impl Period {
    pub fn contains(&self, time: f64) -> bool {
        self.start <= time && time < self.end
    }
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pitch: f64,
    pub period: Period,
}

#[derive(Debug, Clone)]
pub struct Expression(pub Operation);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ChainName {
    String(String),
    Anonymous(usize),
}

impl ChainName {
    pub fn to_string(&self) -> String {
        match *self {
            ChainName::String(ref s) => s.clone(),
            ChainName::Anonymous(i) => format!("anon{:04}", i),
        }
    }
}
impl fmt::Display for ChainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ChainName::String(ref s) => write!(f, "chain: '{}'", s),
            ChainName::Anonymous(num) => write!(f, "anonymous chain #{}", num),
        }
    }
}

#[derive(Debug, Clone)]
pub enum NotesOrId {
    Id(ChainName),
    Notes(Vec<Note>),
}

#[derive(Debug, Clone)]
pub enum ChainLinks {
    Generic(Vec<Expression>),
    OnlyNotes(Vec<NotesOrId>, Period),
}

impl ChainLinks {
    pub fn find_note(&self, time: f64, start_offset: f64, builder: &Builder) -> Option<Note> {
        if let ChainLinks::OnlyNotes(ref notes_or_ids, period) = *self {
            if period.start + start_offset > time || period.end + start_offset <= time {
                None
            } else {
                let mut local_offset = start_offset;
                for notes_or_id in notes_or_ids {
                    match notes_or_id {
                        NotesOrId::Id(ref id) => {
                            if let Some(chain) = builder.find_chain(id) {
                                if let ChainLinks::OnlyNotes(.., period) = chain.links {
                                    let this_offset = local_offset;
                                    local_offset += period.duration();
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
                        NotesOrId::Notes(ref notes) => {
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

#[derive(Debug, Clone)]
pub struct Chain {
    pub name: ChainName,
    pub links: ChainLinks,
    pub play: bool,

    pub true_args: Vec<HashSet<usize>>,
}

#[derive(Debug, Clone)]
struct NameInScope {
    pub name: String,
    pub contents: bool,
}

#[derive(Debug)]
pub struct Builder {
    curr_chains: Vec<Chain>,
    names_in_scope: Vec<NameInScope>,
    pub chains: HashMap<ChainName, Chain>,
    next_anon_chain: usize,
    pub anon_chain_depth: usize,
    pub tempo: f64,
    pub end_time: f64,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            curr_chains: Vec::new(),
            names_in_scope: Vec::new(),
            chains: HashMap::new(),
            next_anon_chain: 0,
            anon_chain_depth: 0,
            tempo: 120.0,
            end_time: 1.0,
        }
    }
    pub fn new_chain(
        &mut self,
        chain_name: Option<String>,
        line: CodeLocation,
    ) -> SonnyResult<ChainName> {
        let return_name = if let Some(cn) = chain_name {
            if self.anon_chain_depth > 0 {
                return Err(Error::new(NamedChainInAnonChain(cn)).on_line(line));
            }
            ChainName::String({
                let final_name = if !self.curr_chains.is_empty() {
                    format!(
                        "{}::{}",
                        if let ChainName::String(ref super_name) =
                            self.curr_chains.last().unwrap().name
                        {
                            super_name.clone()
                        } else {
                            unreachable!()
                        },
                        cn
                    )
                } else {
                    cn
                };
                self.names_in_scope.push(NameInScope {
                    name: final_name.clone(),
                    contents: true,
                });
                final_name
            })
        } else {
            ChainName::Anonymous({
                self.next_anon_chain += 1;
                self.anon_chain_depth += 1;
                self.next_anon_chain - 1
            })
        };
        self.curr_chains.push(Chain {
            name: return_name.clone(),
            links: ChainLinks::Generic(Vec::new()),
            play: false,
            true_args: Vec::new(),
        });
        Ok(return_name)
    }
    pub fn finalize_chain(&mut self) {
        let mut chain = self.curr_chains.pop().expect("No chain to finalize");
        // Turn chain into a Notes chain if necessary
        let mut convert = true;
        let mut only_notes: Vec<NotesOrId> = Vec::new();
        let mut curr_time = 0.0;
        if let ChainLinks::Generic(ref expressions) = chain.links {
            for operation in expressions.iter().map(|expr| &expr.0) {
                match operation {
                    Operation::Operand(Operand::Notes(ref notes)) => {
                        let mut new_notes = Vec::new();
                        for note in notes {
                            new_notes.push(Note {
                                pitch: note.pitch,
                                period: Period {
                                    start: curr_time,
                                    end: curr_time + note.period.duration(),
                                },
                            });
                            curr_time += note.period.duration();
                        }
                        only_notes.push(NotesOrId::Notes(new_notes));
                    }
                    Operation::Operand(Operand::Id(ref notes_chain_name)) => {
                        if let Some(ref notes_chain) = self.find_chain(notes_chain_name) {
                            if let ChainLinks::OnlyNotes(ref _notes_or_ids, period) =
                                notes_chain.links
                            {
                                only_notes.push(NotesOrId::Id(notes_chain_name.clone()));
                                curr_time += period.duration();
                            } else {
                                convert = false;
                                break;
                            }
                        } else {
                            panic!("Unable to find '{}'", notes_chain_name);
                        }
                    }
                    _ => {
                        convert = false;
                        break;
                    }
                }
            }
        }
        if convert {
            chain.links = ChainLinks::OnlyNotes(
                only_notes,
                Period {
                    start: 0.0,
                    end: curr_time,
                },
            );
        }
        if let ChainName::Anonymous(..) = chain.name {
            self.anon_chain_depth -= 1;
        } else {
            self.names_in_scope.pop();
        }
        self.chains.insert(chain.name.clone(), chain);
    }
    pub fn find_chain(&self, name: &ChainName) -> Option<&Chain> {
        match *name {
            ChainName::Anonymous(..) => self.chains.get(name),
            ChainName::String(ref name_str) => {
                if let Some(chain) = self.chains.get(name) {
                    Some(chain)
                } else {
                    for name_in_scope in &self.names_in_scope {
                        if name_in_scope.contents {
                            let test_name = format!("{}::{}", name_in_scope.name, name_str);
                            if let Some(ref chain) = self.chains.get(&ChainName::String(test_name))
                            {
                                return Some(chain);
                            }
                        } else {
                            if &name_in_scope.name.split("::").last().unwrap() == name_str {
                                return self.chains
                                    .get(&ChainName::String(name_in_scope.name.clone()));
                            }
                        }
                    }
                    None
                }
            }
        }
    }
    pub fn play_chain(&mut self) {
        if let Some(ref mut chain) = self.curr_chains.last_mut() {
            chain.play = true;
        } else {
            panic!("No current chain to set to play");
        }
    }
    pub fn new_expression(&mut self, expression: Expression) {
        if let Some(chain) = self.curr_chains.last_mut() {
            if let ChainLinks::Generic(ref mut expressions) = chain.links {
                expressions.push(expression);
            } else {
                panic!("Cannot add expression to Notes chain");
            }
        } else {
            panic!("No current chain to add expressions to");
        }
    }
}
