use std::collections::HashMap;
use std::f64;

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
    pub fn forever() -> Period {
        Period {
            start: 0.0,
            end: f64::MAX,
        }
    }
    pub fn duration(&self) -> f64 {
        self.end - self.start
    }
}

#[derive(Debug, Clone)]
pub struct Note {
    pub pitch: f64,
    pub period: Period,
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub operation: Operation,
}

impl Expression {
    pub fn new(operation: Operation) -> Expression {
        Expression { operation }
    }
}

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

#[derive(Debug, Clone)]
pub struct Chain {
    pub name: ChainName,
    pub links: Vec<Expression>,
    pub period: Period,
    pub play: bool,
}

#[derive(Debug)]
pub struct Builder {
    curr_chains: Vec<Chain>,
    pub chains: HashMap<ChainName, Chain>,
    next_anon_chain: usize,
    pub tempo: f64,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            curr_chains: Vec::new(),
            chains: HashMap::new(),
            next_anon_chain: 0,
            tempo: 120.0,
        }
    }
    pub fn new_chain(&mut self) {
        self.curr_chains.push(Chain {
            name: ChainName::String(String::new()),
            links: Vec::new(),
            period: Period::forever(),
            play: false,
        });
    }
    pub fn finalize_chain(&mut self, name: Option<String>) -> ChainName {
        let mut chain = self.curr_chains.pop().expect("No chain to finalize");
        let chain_name;
        // Fix chain period endings of notes chains
        if let Operation::Operand(Operand::Notes(ref notes)) = chain.links[0].operation {
            if let Some(ref note) = notes.last() {
                if chain.period.end == f64::MAX {
                    chain.period.end = note.period.end;
                }
            } else {
                panic!("Notes are empty");
            }
        }
        // Assign name and insert into chains map
        if let Some(n) = name {
            chain.name = ChainName::String(n.clone());
            chain_name = chain.name.clone();
            self.chains.insert(ChainName::String(n), chain);
        } else {
            chain.name = ChainName::Anonymous(self.next_anon_chain);
            chain_name = chain.name.clone();
            self.chains
                .insert(ChainName::Anonymous(self.next_anon_chain), chain);
            self.next_anon_chain += 1;
        }
        chain_name
    }
    pub fn chain_period(&mut self, period: Period) {
        if let Some(ref mut chain) = self.curr_chains.last_mut() {
            chain.period = period;
        } else {
            panic!("No current chain to set period");
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
        if let Some(ref mut chain) = self.curr_chains.last_mut() {
            chain.links.push(expression);
        } else {
            panic!("No current chain to add expressions to");
        }
    }
    pub fn find_chain(&self, name: &ChainName) -> Option<&Chain> {
        if self.chains.contains_key(name) {
            self.chains.get(name)
        } else {
            self.chains.values().find(|c| &c.name == name)
        }
    }
}
