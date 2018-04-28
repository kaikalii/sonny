use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Operand {
    Num(f64),
    Id(String),
    BackLink(Option<String>, Vec<usize>),
    Time,
    SampleRate,
    Operation(Box<Operation>),
}

#[derive(Debug, Clone)]
pub enum Operation {
    Add(Operand, Operand),
    Substract(Operand, Operand),
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
    None(Operand),
}

#[derive(Debug, Clone)]
pub enum Time {
    Absolute(f64),
    Start,
    End,
}

#[derive(Debug, Clone)]
pub struct Period {
    pub start: Time,
    pub end: Time,
}

#[derive(Debug, Clone)]
pub struct Note {
    pub pitch: f64,
    pub period: Period,
}

#[derive(Debug, Clone)]
enum LinkBody {
    Expression(Operation),
    Notes(Vec<Note>),
}

#[derive(Debug, Clone)]
struct Link {
    body: LinkBody,
    period: Period,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum LinkName {
    String(String),
    Anonymous,
}

#[derive(Debug, Clone)]
struct Chain {
    pub name: Option<String>,
    pub link_names: Vec<LinkName>,
    pub play: bool,
}

#[derive(Debug)]
pub struct Builder {
    chains: Vec<Chain>,
    links: HashMap<LinkName, Link>,
}

impl Builder {
    pub fn new() -> Builder {
        Builder {
            chains: Vec::new(),
            links: HashMap::new(),
        }
    }
    pub fn new_chain(&mut self) {
        self.chains.push(Chain {
            name: None,
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
            .name = Some(name);
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
            LinkName::Anonymous
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
            Link {
                body: LinkBody::Expression(top_op),
                period: period,
            },
        );
    }
    pub fn new_notes(&mut self, name: Option<String>, period: Period, notes: Vec<Note>) {
        let link_name = if let Some(n) = name {
            LinkName::String(n.clone())
        } else {
            LinkName::Anonymous
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
            Link {
                body: LinkBody::Notes(notes),
                period: period,
            },
        );
    }
}
