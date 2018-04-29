use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::f64;

#[derive(Debug, Clone)]
pub enum Operand {
    Num(f64),
    Id(String),
    BackLink(usize),
    BackChain(usize),
    Time,
    Operation(Box<Operation>),
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
    NoOperation(Operand),
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
pub enum LinkBody {
    Notes(Vec<Note>),
    Expression(Operation),
}

#[derive(Debug, Clone)]
pub struct Link {
    pub body: LinkBody,
    pub period: Period,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ChainName {
    String(String),
    Anonymous(usize),
}

#[derive(Debug, Clone)]
pub struct Chain {
    pub name: ChainName,
    pub links: Vec<Link>,
    pub play: bool,
}

#[derive(Debug)]
pub struct Builder {
    curr_chain: Option<Chain>,
    pub chains: HashMap<ChainName, RefCell<Chain>>,
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
            self.chains
                .insert(ChainName::String(n), RefCell::new(chain));
        } else {
            chain.name = ChainName::Anonymous(self.next_anon_chain);
            self.chains.insert(
                ChainName::Anonymous(self.next_anon_chain),
                RefCell::new(chain),
            );
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
    pub fn new_expression(&mut self, period: Period, top_op: Operation) {
        if let Some(ref mut chain) = self.curr_chain {
            chain.links.push(Link {
                body: LinkBody::Expression(top_op),
                period: period,
            });
        } else {
            panic!("No current chain add expressions to");
        }
    }
    pub fn new_notes(&mut self, period: Period, notes: Vec<Note>) {
        if let Some(ref mut chain) = self.curr_chain {
            chain.links.push(Link {
                body: LinkBody::Notes(notes),
                period: period,
            });
        } else {
            panic!("No current chain add notes to");
        }
    }
    pub fn chain(&self, name: &ChainName) -> Ref<Chain> {
        self.chains[name].borrow()
    }
    pub fn evaluate_operand(
        &self,
        stack: &mut Vec<(ChainName, usize)>,
        op: &Operand,
        time: f64,
    ) -> f64 {
        use self::Operand::*;
        match *op {
            Num(x) => x,
            Id(ref id) => {
                println!("id: {}", id);
                if self.chains.contains_key(&ChainName::String(id.clone())) {
                    if self.chains[&ChainName::String(id.clone())]
                        .try_borrow()
                        .is_ok()
                    {
                        stack.push((ChainName::String(id.clone()), 0));
                        let result = self.evaluate_chain(stack, time);
                        stack.pop();
                        result
                    } else {
                        panic!("Detected recursive chain: \"{}\"", id);
                    }
                } else {
                    panic!("Unkown id: \"{}\"", id);
                }
            }
            BackLink(num) => {
                println!("backlink: {}", num);
                stack.last_mut().unwrap().1 += num + 1;
                let result = self.evaluate_link(stack, time);
                stack.last_mut().unwrap().1 -= num + 1;
                result
            }
            BackChain(num) => {
                println!("backchain: {}", num);
                let mut upper_chains = Vec::new();
                for _ in 0..(num + 1) {
                    upper_chains.push(stack.pop().unwrap());
                }
                stack.last_mut().unwrap().1 += num + 1;
                let result = self.evaluate_link(stack, time);
                stack.last_mut().unwrap().1 -= num + 1;
                for _ in 0..(num + 1) {
                    stack.push(upper_chains.pop().unwrap());
                }
                result
            }
            Time => time,
            Operation(ref operation) => self.evaluate_operation(stack, operation, time),
        }
    }
    pub fn evaluate_operation(
        &self,
        stack: &mut Vec<(ChainName, usize)>,
        operation: &Operation,
        time: f64,
    ) -> f64 {
        use self::Operation::*;
        let (a, b) = match *operation {
            Add(ref a, ref b)
            | Subtract(ref a, ref b)
            | Multiply(ref a, ref b)
            | Divide(ref a, ref b)
            | Remainder(ref a, ref b)
            | Power(ref a, ref b) => (a, Some(b)),
            Negate(ref a) | Sine(ref a) | Cosine(ref a) | Ceiling(ref a) | Floor(ref a)
            | AbsoluteValue(ref a) | NoOperation(ref a) => (a, None),
        };

        let x = self.evaluate_operand(stack, a, time);
        let y = b.map(|bb| self.evaluate_operand(stack, bb, time));
        match *operation {
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
            NoOperation(..) => x,
        }
    }
    pub fn evaluate_link(&self, stack: &mut Vec<(ChainName, usize)>, time: f64) -> f64 {
        println!(
            "Evaluatiing backlink {} of chain {:?}",
            stack.last().unwrap().1,
            stack.last().unwrap().0
        );
        let mut upper_chains = Vec::new();
        let mut link = None;
        loop {
            link = self.chain(&stack.last().unwrap().0)
                .links
                .iter()
                .rev()
                .skip(stack.last().unwrap().1)
                .next();
            if link.is_some() {
                break;
            } else {

            }
        }

        let link = link.unwrap();
        match link.body {
            LinkBody::Notes(ref notes) => {
                for note in notes {
                    if note.period.contains(Time::Absolute(time)) {
                        return note.pitch;
                    }
                }
                panic!("Unable to get frequency from notes at time {}", time);
            }
            LinkBody::Expression(ref operation) => self.evaluate_operation(stack, operation, time),
        }
    }
    pub fn evaluate_chain(&self, stack: &mut Vec<(ChainName, usize)>, time: f64) -> f64 {
        if !self.chain(&stack.last().unwrap().0).links.is_empty() {
            self.evaluate_link(stack, time)
        } else {
            panic!("Tried to evaluate empty chain");
        }
    }
}
