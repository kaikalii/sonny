use std::collections::{HashMap, HashSet};
use std::f64;

use builder::*;

#[derive(Debug, Clone)]
pub struct FunctionDef {
    pub true_args: Vec<HashSet<usize>>,
    pub chain: Chain,
}

#[derive(Debug)]
pub struct Functions {
    pub builder: Builder,
    pub functions: HashMap<ChainName, FunctionDef>,
}

impl Functions {
    pub fn new(builder: Builder) -> Functions {
        let mut f = Functions {
            builder,
            functions: HashMap::new(),
        };
        f.make_functions();
        // println!(
        //     "functions: {:?}",
        //     (f.functions
        //         .iter()
        //         .map(|f| (f.0.clone(), f.1.true_args.clone()))
        //         .collect::<Vec<(ChainName, Vec<HashSet<usize>>)>>())
        // );
        f
    }
    fn collect_args(&mut self, expression: &Expression) -> HashSet<usize> {
        let mut args = HashSet::new();
        let operands = expression.0.operands();
        use Operand::*;
        match *operands.0 {
            Id(ref id) => {
                if !self.functions.contains_key(id) {
                    if !self.builder.chains.contains_key(id) {
                        panic!("No known id: {:?}", id);
                    }
                    let chain = self.builder.chains[id].clone();
                    self.make_function(chain);
                }
                if !self.functions[id].true_args.is_empty() {
                    args = args.union(self.functions[id].true_args.last().unwrap())
                        .cloned()
                        .collect();
                }
            }
            Expression(ref expr) => {
                args = args.union(&self.collect_args(&expr)).cloned().collect();
            }
            BackLink(num) => {
                args.insert(num);
            }
            _ => (),
        }
        if let Some(op) = operands.1 {
            match *op {
                Id(ref id) => {
                    if !self.functions.contains_key(id) {
                        let chain = self.builder.chains[id].clone();
                        self.make_function(chain);
                    }
                    if !self.functions[id].true_args.is_empty() {
                        args = args.union(self.functions[id].true_args.last().unwrap())
                            .cloned()
                            .collect();
                    }
                }
                Expression(ref expr) => {
                    args = args.union(&self.collect_args(&expr)).cloned().collect();
                }
                BackLink(num) => {
                    args.insert(num);
                }
                _ => (),
            }
        }
        args
    }

    fn make_function(&mut self, chain: Chain) {
        let mut true_args: Vec<HashSet<usize>> = Vec::new();
        if let ChainLinks::Generic(ref expressions) = chain.links {
            for (i, expression) in expressions.iter().enumerate() {
                let temp_args = self.collect_args(expression);
                let mut final_args: HashSet<usize> = HashSet::new();
                for arg in temp_args {
                    if (i as i32 - arg as i32) < 0 {
                        final_args.insert((i as i32 - arg as i32).abs() as usize);
                    } else {
                        for &prev_arg in &true_args[i - arg] {
                            final_args.insert(prev_arg);
                        }
                    }
                }
                true_args.push(final_args);
            }
        }
        self.functions.insert(
            chain.name.clone(),
            FunctionDef {
                true_args,
                chain: chain.clone(),
            },
        );
    }

    fn make_functions(&mut self) {
        for chain in self.builder
            .chains
            .values()
            .filter(|c| c.play)
            .cloned()
            .collect::<Vec<Chain>>()
        {
            if !self.functions.contains_key(&chain.name) {
                self.make_function(chain);
            }
        }
    }

    fn evaluate_operand(
        &self,
        operand: &Operand,
        name: &ChainName,
        args: &[f64],
        time: f64,
        depth: usize,
    ) -> f64 {
        use Operand::*;
        match *operand {
            Num(x) => x,
            Id(ref id) => self.evaluate_function(id, args, time, depth),
            Property(ref id, property) => if let Some(chain) = self.builder.chains.get(id) {
                if let ChainLinks::OnlyNotes(..) = chain.links {
                    if let Some(note) = chain.links.find_note(time, 0.0, &self.builder.chains) {
                        use builder::Property::*;
                        match property {
                            Start => note.period.start,
                            End => note.period.end,
                            Duration => note.period.duration(),
                        }
                    } else {
                        0.0
                    }
                } else {
                    panic!("Reference chain is not a note chain");
                }
            } else {
                panic!("Unknown id {:?}", id)
            },
            Time => time,
            BackLink(num) => args[num - 1],
            Notes(ref notes) => {
                let mut result = 0.0;
                for note in notes {
                    if note.period.contains(time) {
                        result = note.pitch;
                        break;
                    }
                }
                result
            }
            Expression(ref expression) => {
                self.evaluate_expression(expression, name, args, time, depth)
            }
        }
    }

    fn evaluate_expression(
        &self,
        expression: &Expression,
        name: &ChainName,
        args: &[f64],
        time: f64,
        depth: usize,
    ) -> f64 {
        use self::Operation::*;
        let (a, b) = expression.0.operands();
        let x = self.evaluate_operand(a, name, args, time, depth);
        let y = b.map(|bb| self.evaluate_operand(bb, name, args, time, depth));
        match expression.0 {
            Add(..) => x + y.expect("failed to unwrap y in add"),
            Subtract(..) => x - y.expect("failed to unwrap y in subtract"),
            Multiply(..) => x * y.expect("failed to unwrap y in multiply"),
            Divide(..) => x / y.expect("failed to unwrap y in divide"),
            Remainder(..) => x % y.expect("failed to unwrap y in remainder"),
            Power(..) => x.powf(y.expect("failed to unwrap y in min")),
            Min(..) => x.min(y.expect("failed to unwrap y in min")),
            Max(..) => x.max(y.expect("failed to unwrap y in max")),
            Negate(..) => -x,
            Sine(..) => x.sin(),
            Cosine(..) => x.cos(),
            Ceiling(..) => x.ceil(),
            Floor(..) => x.floor(),
            AbsoluteValue(..) => x.abs(),
            Logarithm(..) => x.log(f64::consts::E),
            Operand(..) => x,
        }
    }

    pub fn evaluate_function(
        &self,
        name: &ChainName,
        args: &[f64],
        time: f64,
        depth: usize,
    ) -> f64 {
        if let Some(function) = self.functions.get(name) {
            match function.chain.links {
                ChainLinks::Generic(ref expressions) => {
                    let mut results = Vec::new();
                    for (_i, expression) in expressions.iter().enumerate() {
                        let mut these_args = Vec::new();
                        for &r in results.iter().rev() {
                            these_args.push(r);
                        }
                        for &a in args {
                            these_args.push(a);
                        }
                        results.push(self.evaluate_expression(
                            expression,
                            name,
                            &these_args,
                            time,
                            depth + 6,
                        ));
                    }
                    *results.last().expect("generic chain gave no last result")
                }
                ChainLinks::OnlyNotes(..) => function
                    .chain
                    .links
                    .find_note(time, 0.0, &self.builder.chains)
                    .map(|n| n.pitch)
                    .unwrap_or(0.0),
            }
        } else {
            panic!("No function named '{}'", name);
        }
    }
}
