pub mod evaluate;
pub mod variable;

use std::collections::HashMap;
use std::f64;
use std::fmt;

use error::{ErrorSpec::*, *};
use lexer::CodeLocation;

use self::variable::*;

// Different types of Notes properties
#[derive(Debug, Clone, Copy)]
pub enum Property {
    Start,
    End,
    Duration,
}

// Different types of operands
#[derive(Debug, Clone)]
pub enum Operand {
    Var(Variable),
    Id(ChainName),
    Property(ChainName, Property),
    BackLink(usize),
    Time,
    Notes(Vec<Note>),
    Expression(Box<Expression>),
}

// Different types of operations
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
    LessThan(Operand, Operand),
    GreaterThan(Operand, Operand),
    LessThanOrEqual(Operand, Operand),
    GreaterThanOrEqual(Operand, Operand),
    Equal(Operand, Operand),
    NotEqual(Operand, Operand),
    Or(Operand, Operand),
    And(Operand, Operand),
    Negate(Operand),
    Sine(Operand),
    Cosine(Operand),
    Floor(Operand),
    Ceiling(Operand),
    AbsoluteValue(Operand),
    Logarithm(Operand),
    Operand(Operand),
    Ternary(Operand, Operand, Operand),
}

impl Operation {
    // return a pair of the first and optinal second operand of the operation
    pub fn operands(&self) -> (&Operand, Option<&Operand>, Option<&Operand>) {
        use self::Operation::*;
        match *self {
            Add(ref a, ref b)
            | Subtract(ref a, ref b)
            | Multiply(ref a, ref b)
            | Divide(ref a, ref b)
            | Remainder(ref a, ref b)
            | Power(ref a, ref b)
            | Min(ref a, ref b)
            | Max(ref a, ref b)
            | LessThan(ref a, ref b)
            | GreaterThan(ref a, ref b)
            | LessThanOrEqual(ref a, ref b)
            | GreaterThanOrEqual(ref a, ref b)
            | Equal(ref a, ref b)
            | NotEqual(ref a, ref b)
            | Or(ref a, ref b)
            | And(ref a, ref b) => (a, Some(b), None),
            Negate(ref a) | Sine(ref a) | Cosine(ref a) | Ceiling(ref a) | Floor(ref a)
            | AbsoluteValue(ref a) | Logarithm(ref a) | Operand(ref a) => (a, None, None),
            Ternary(ref a, ref b, ref c) => (a, Some(b), Some(c)),
        }
    }
}

// Defines a period of time with a start and an end
#[derive(Debug, Clone, Copy)]
pub struct Period {
    pub start: f64,
    pub end: f64,
}

impl Period {
    // Does a time fall within the period?
    pub fn contains(&self, time: f64) -> bool {
        self.start <= time && time < self.end
    }

    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
}

// A pitch with a period
#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub pitch: f64,
    pub period: Period,
}

// Expression may have more in the future, but for now,
// they just hold a top-lvel operation
#[derive(Debug, Clone)]
pub struct Expression(pub Operation);

// A name that a chain can have
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ChainName {
    Scoped(String),
    Anonymous(usize),
}

impl ChainName {
    // Convert the ChainName to a string for output file names
    pub fn to_string(&self) -> String {
        match *self {
            ChainName::Scoped(ref s) => s.clone(),
            ChainName::Anonymous(i) => format!("anon{:04}", i),
        }
    }
}

// Format the ChainName for error messages
impl fmt::Display for ChainName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ChainName::Scoped(ref s) => write!(f, "chain: '{}'", s),
            ChainName::Anonymous(num) => write!(f, "anonymous chain #{}", num),
        }
    }
}

// Can either be a vec of notes or the ChainName of a chain with
// only OnlyNotes links.
#[derive(Debug, Clone)]
pub enum NotesOrId {
    Id(ChainName),
    Notes(Vec<Note>),
}

// The two basic types a link in a chain can have
#[derive(Debug, Clone)]
pub enum ChainLinks {
    // Generic Links can have anything in them. All links are initially
    // constructed as generic.
    Generic(Vec<Expression>),
    // Links that contains only NotesOrIds. Traversing the tree down
    // from an OnlyNotes chain will always only reveal more notes.
    // Generic links with only notes are converted to this type of links
    // when the chain is finalized. The period from the first to last
    // notes is kept track of.
    OnlyNotes(Vec<NotesOrId>, Period),
}

// A Chain with a name, links, and whether or not it should be output
#[derive(Debug, Clone)]
pub struct Chain {
    pub name: ChainName,
    pub links: ChainLinks,
    pub play: bool,
}

// A name that is in scope with a marker telling whether it is the
// name itself or its contents whish are actually in scope.
// i.e. "use gen" vs "use gen::*"
#[derive(Debug, Clone)]
pub struct NameInScope {
    pub name: String,
    pub contents: bool,
}

// The main builder with manages the initialization, contruction,
// and finalization of chains. Most of its methods are called by
// the parser.
#[derive(Debug)]
pub struct Builder {
    // The chains that are currently being built and are not yet finalized
    curr_chains: Vec<Chain>,
    // A list of the names in scope
    pub names_in_scope: Vec<NameInScope>,
    // The chains that are finalized.
    pub chains: HashMap<ChainName, Chain>,
    // The number of the next anonymous chain
    next_anon_chain: usize,
    // How many nested anonymous chains there are currently. As long as
    // this number is nonzero, named chains cannot be created.
    pub anon_chain_depth: usize,
    // The current tempo of the audio. Used for correctly assigning
    // periods to notes
    pub tempo: f64,
    // The time at which the audio is set to stop. Will be overridden
    // by any notes which are longer
    pub end_time: f64,
}

impl Builder {
    // Makes a new Builder
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
    // Initializes a new chain
    pub fn new_chain(
        &mut self,
        chain_name: Option<String>,
        line: CodeLocation,
    ) -> SonnyResult<ChainName> {
        // Figure out what it should be named
        let return_name = if let Some(cn) = chain_name {
            if self.anon_chain_depth > 0 {
                return Err(Error::new(NamedChainInAnonChain(cn)).on_line(line));
            }
            ChainName::Scoped({
                let final_name = if !self.curr_chains.is_empty() && !self.names_in_scope.is_empty()
                {
                    format!(
                        "{}::{}",
                        self.names_in_scope.last().expect("no names in scope").name,
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
        // Make sure another chain does not already have that name
        if self.find_chain(&return_name).is_some() {
            return Err(Error::new(ChainRedeclaration(return_name.clone())));
        }
        // Push the chain onto the current chains
        self.curr_chains.push(Chain {
            name: return_name.clone(),
            links: ChainLinks::Generic(Vec::new()),
            play: false,
        });
        Ok(return_name)
    }
    // Cleans up a chain's state when its construction is finished
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
        // Conver the chain's links into OnlyNotes if necessary
        if convert {
            chain.links = ChainLinks::OnlyNotes(
                only_notes,
                Period {
                    start: 0.0,
                    end: curr_time,
                },
            );
        }
        // Do something depending on what kind of hame the chain has
        if let ChainName::Anonymous(..) = chain.name {
            self.anon_chain_depth -= 1;
        } else {
            self.names_in_scope.pop();
        }
        // Insert the chain
        self.chains.insert(chain.name.clone(), chain);
    }
    // Finds a chain with the given name. In the chains map, chains are named
    // with their full scoped names. This function finds a chain with only
    // the last part of the scoped name, given that it would actually be in scope.
    pub fn find_chain(&self, name: &ChainName) -> Option<&Chain> {
        match *name {
            ChainName::Anonymous(..) => self.chains.get(name),
            ChainName::Scoped(ref name_str) => {
                if let Some(chain) = self.chains.get(name) {
                    Some(chain)
                } else {
                    for name_in_scope in &self.names_in_scope {
                        if name_in_scope.contents {
                            let test_name = format!("{}::{}", name_in_scope.name, name_str);
                            if let Some(ref chain) = self.chains.get(&ChainName::Scoped(test_name))
                            {
                                return Some(chain);
                            }
                        } else {
                            if &name_in_scope.name.split("::").last().unwrap() == name_str {
                                return self.chains
                                    .get(&ChainName::Scoped(name_in_scope.name.clone()));
                            }
                        }
                    }
                    None
                }
            }
        }
    }
    // Sets a chain to be one that is output.
    pub fn play_chain(&mut self) {
        if let Some(ref mut chain) = self.curr_chains.last_mut() {
            chain.play = true;
        } else {
            panic!("No current chain to set to play");
        }
    }
    // Adds a new expression to the most recently created chain
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
