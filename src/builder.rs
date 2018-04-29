use std::collections::HashMap;
use std::f64;

#[derive(Debug, Clone)]
pub enum Operand {
    Num(f64),
    Id(String),
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
    Negate(Operand),
    Sine(Operand),
    Cosine(Operand),
    Floor(Operand),
    Ceiling(Operand),
    AbsoluteValue(Operand),
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
            | Power(ref a, ref b) => (a, Some(b)),
            Negate(ref a) | Sine(ref a) | Cosine(ref a) | Ceiling(ref a) | Floor(ref a)
            | AbsoluteValue(ref a) | Operand(ref a) => (a, None),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Time {
    Absolute(f64),
    Start,
    End,
}

impl Time {
    pub fn to_f64(&self) -> f64 {
        match *self {
            Time::Absolute(a) => a,
            Time::Start => 0.0,
            Time::End => f64::MAX,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Period {
    pub start: Time,
    pub end: Time,
}

impl Period {
    pub fn contains(&self, time: Time) -> bool {
        time.to_f64().ge(&self.start.to_f64())
            && (time.to_f64().lt(&self.end.to_f64()) || self.end == Time::End && time == Time::End)
    }
    pub fn forever() -> Period {
        Period {
            start: Time::Start,
            end: Time::End,
        }
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
    curr_chain: Option<Chain>,
    pub chains: HashMap<ChainName, Chain>,
    next_anon_chain: usize,
    pub tempo: f64,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            curr_chain: None,
            chains: HashMap::new(),
            next_anon_chain: 0,
            tempo: 120.0,
        }
    }
    pub fn new_chain(&mut self) {
        self.curr_chain = Some(Chain {
            name: ChainName::String(String::new()),
            links: Vec::new(),
            period: Period::forever(),
            play: false,
        });
    }
    pub fn finalize_chain(&mut self, name: Option<String>) {
        let mut chain = self.curr_chain.take().expect("No chain to finalize");
        if let Some(n) = name {
            chain.name = ChainName::String(n.clone());
            self.chains.insert(ChainName::String(n), chain);
        } else {
            chain.name = ChainName::Anonymous(self.next_anon_chain);
            self.chains
                .insert(ChainName::Anonymous(self.next_anon_chain), chain);
            self.next_anon_chain += 1;
        }
    }
    pub fn chain_period(&mut self, period: Period) {
        if let Some(ref mut chain) = self.curr_chain {
            chain.period = period;
        } else {
            panic!("No current chain to set period");
        }
    }
    pub fn play_chain(&mut self) {
        if let Some(ref mut chain) = self.curr_chain {
            chain.play = true;
        } else {
            panic!("No current chain to set to play");
        }
    }
    pub fn new_expression(&mut self, expression: Expression) {
        if let Some(ref mut chain) = self.curr_chain {
            chain.links.push(expression);
        } else {
            panic!("No current chain add expressions to");
        }
    }
}
