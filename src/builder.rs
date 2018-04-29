use std::collections::HashMap;
use std::f64;

#[derive(Debug, Clone)]
pub enum Operand {
    Num(f64),
    Id(String),
    BackLink(usize),
    Time,
    Notes(Vec<Note>),
    Chain(Chain, usize),
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
    pub fn operands_mut(&mut self) -> (&mut Operand, Option<&mut Operand>) {
        use self::Operation::*;
        match *self {
            Add(ref mut a, ref mut b)
            | Subtract(ref mut a, ref mut b)
            | Multiply(ref mut a, ref mut b)
            | Divide(ref mut a, ref mut b)
            | Remainder(ref mut a, ref mut b)
            | Power(ref mut a, ref mut b) => (a, Some(b)),
            Negate(ref mut a)
            | Sine(ref mut a)
            | Cosine(ref mut a)
            | Ceiling(ref mut a)
            | Floor(ref mut a)
            | AbsoluteValue(ref mut a)
            | Operand(ref mut a) => (a, None),
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
            Time::End => panic!("Period cannot begin at end"),
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
}

#[derive(Debug, Clone)]
pub struct Note {
    pub pitch: f64,
    pub period: Period,
}

#[derive(Debug, Clone)]
pub struct Expression {
    pub operation: Operation,
    pub period: Period,
}

impl Expression {
    pub fn new(operation: Operation, period: Period) -> Expression {
        Expression { operation, period }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ChainName {
    String(String),
    Anonymous(usize),
}

#[derive(Debug, Clone)]
pub struct Chain {
    pub name: ChainName,
    pub links: Vec<Expression>,
    pub play: bool,
}

#[derive(Debug)]
pub struct Builder {
    curr_chain: Option<Chain>,
    pub chains: HashMap<ChainName, Chain>,
    next_anon_chain: usize,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            curr_chain: None,
            chains: HashMap::new(),
            next_anon_chain: 0,
        }
    }
    pub fn new_chain(&mut self) {
        self.curr_chain = Some(Chain {
            name: ChainName::String(String::new()),
            links: Vec::new(),
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
    pub fn evaluate_operand(&self, op: &Operand, time: f64) -> f64 {
        use self::Operand::*;
        match *op {
            Num(x) => x,
            Time => time,
            Expression(ref expression) => self.evaluate_expression(expression, time),
            _ => panic!("Unsimplified operand"),
        }
    }
    pub fn evaluate_expression(&self, expression: &Expression, time: f64) -> f64 {
        use self::Operation::*;
        let (a, b) = expression.operation.operands();
        let x = self.evaluate_operand(a, time);
        let y = b.map(|bb| self.evaluate_operand(bb, time));
        match expression.operation {
            Add(..) => x + y.unwrap(),
            Subtract(..) => x - y.unwrap(),
            Multiply(..) => x * y.unwrap(),
            Divide(..) => x / y.unwrap(),
            Remainder(..) => x % y.unwrap(),
            Power(..) => x.powf(y.unwrap()),
            Negate(..) => -x,
            Sine(..) => x.sin(),
            Cosine(..) => x.cos(),
            Ceiling(..) => x.ceil(),
            Floor(..) => x.floor(),
            AbsoluteValue(..) => x.abs(),
            Operand(..) => x,
        }
    }
    pub fn eliminate_backlinks(&mut self, chain: &mut Chain, backlink: usize) -> bool {
        let operands = chain
            .links
            .iter_mut()
            .rev()
            .skip(backlink)
            .next()
            .expect("Tried to eleminate backlinks of empty chain")
            .operation
            .operands_mut();
        use self::Operand::*;
        let mut result = false;
        *operands.0 = match operands.0 {
            BackLink(num) => Operand::Chain(chain.clone(), backlink + num.clone()),
            Chain(chain, backlink) => {
                // if chain.links.len() == 1
            }
            Operation(expr) => {}
            _ => (),
        };
        result
    }
}
