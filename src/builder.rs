use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::f64;

#[derive(Debug, Clone)]
pub enum Operand {
    Num(f64),
    Id(String),
    BackLink(Option<String>, Vec<usize>),
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
pub enum LinkName {
    String(String),
    Anonymous(usize),
}

#[derive(Debug, Clone)]
pub struct Chain {
    pub name: LinkName,
    pub link_names: Vec<LinkName>,
    pub play: bool,
}

#[derive(Debug)]
pub struct Builder {
    pub chains: Vec<Chain>,
    pub links: HashMap<LinkName, RefCell<Link>>,
    next_anon_link: usize,
    next_anon_chain: usize,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            chains: Vec::new(),
            links: HashMap::new(),
            next_anon_link: 0,
            next_anon_chain: 0,
        }
    }
    pub fn new_chain(&mut self) {
        self.chains.push(Chain {
            name: {
                self.next_anon_chain += 1;
                LinkName::Anonymous(self.next_anon_chain - 1)
            },
            link_names: Vec::new(),
            play: false,
        });
    }
    pub fn name_chain(&mut self, name: String) {
        self.chains
            .iter_mut()
            .rev()
            .next()
            .expect("Attempted to name non-existant first chain")
            .name = LinkName::String(name);
        self.next_anon_chain -= 1;
    }
    pub fn play_chain(&mut self) {
        self.chains
            .iter_mut()
            .rev()
            .next()
            .expect("Attempted to set play of non-existant first chain")
            .play = true;
    }
    pub fn new_expression(&mut self, name: Option<String>, period: Period, top_op: Operation) {
        let link_name = if let Some(n) = name {
            LinkName::String(n.clone())
        } else {
            LinkName::Anonymous({
                self.next_anon_link += 1;
                self.next_anon_link - 1
            })
        };
        self.chains
            .iter_mut()
            .rev()
            .next()
            .expect("Attempted to insert into non-existant first chain")
            .link_names
            .push(link_name.clone());
        self.links.insert(
            link_name,
            RefCell::new(Link {
                body: LinkBody::Expression(top_op),
                period: period,
            }),
        );
    }
    pub fn new_notes(&mut self, name: Option<String>, period: Period, notes: Vec<Note>) {
        let link_name = if let Some(n) = name {
            LinkName::String(n.clone())
        } else {
            LinkName::Anonymous({
                self.next_anon_link += 1;
                self.next_anon_link - 1
            })
        };
        self.chains
            .iter_mut()
            .rev()
            .next()
            .expect("Attempted to insert into non-existant first chain")
            .link_names
            .push(link_name.clone());
        self.links.insert(
            link_name,
            RefCell::new(Link {
                body: LinkBody::Notes(notes),
                period: period,
            }),
        );
    }
    pub fn evaluate_link(&self, link: RefMut<Link>, time: f64) -> f64 {
        match link.body {
            LinkBody::Notes(ref notes) => {
                for note in notes {
                    if note.period.contains(Time::Absolute(time)) {
                        return note.pitch;
                    }
                }
                panic!("Unable to get frequency from notes at time {}", time);
            }
            LinkBody::Expression(ref operation) => {
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

                let (mut x, mut y) = (0.0, Some(0.0));
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
        }
    }
}
