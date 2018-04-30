use std::collections::{HashMap, HashSet};
use std::f64;

use builder::{self, *};

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
        let operands = expression.operation.operands();
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
                args = args.union(self.functions[id].true_args.last().unwrap())
                    .cloned()
                    .collect();
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
                    args = args.union(self.functions[id].true_args.last().unwrap())
                        .cloned()
                        .collect();
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
        for (i, expression) in chain.links.iter().enumerate() {
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
            Time => time,
            BackLink(num) => args[num - 1],
            Notes(ref notes) => {
                let mut result = 0.0;
                for note in notes {
                    if note.period.contains(builder::Time::Absolute(time)) {
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
        let (a, b) = expression.operation.operands();
        let x = self.evaluate_operand(a, name, args, time, depth);
        let y = b.map(|bb| self.evaluate_operand(bb, name, args, time, depth));
        match expression.operation {
            Add(..) => x + y.unwrap(),
            Subtract(..) => x - y.unwrap(),
            Multiply(..) => x * y.unwrap(),
            Divide(..) => x / y.unwrap(),
            Remainder(..) => x % y.unwrap(),
            Power(..) => x.powf(y.unwrap()),
            Min(..) => x.min(y.unwrap()),
            Max(..) => x.max(y.unwrap()),
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
        if time == 0.0 {
            // println!(
            //     "{}Calling function {:?}",
            //     (0..depth).map(|_| ' ').collect::<String>(),
            //     name
            // );
            // println!(
            //     "{}  with args: {:?}",
            //     (0..depth).map(|_| ' ').collect::<String>(),
            //     args
            // );
        }
        if self.functions[name]
            .chain
            .period
            .contains(Time::Absolute(time))
        {
            let mut results = Vec::new();
            for (_i, expression) in self.functions[name].chain.links.iter().enumerate() {
                // if time == 0.0 {
                //     println!(
                //         "{}    expression: {}",
                //         (0..depth).map(|_| ' ').collect::<String>(),
                //         i
                //     );
                // }
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
            // if time == 0.0 {
            //     print!("{}", (0..depth).map(|_| ' ').collect::<String>());
            //     println!("result: {}", results.last().unwrap());
            // }
            *results.last().unwrap()
        } else {
            // if time == 0.0 {
            //     print!("{}", (0..depth).map(|_| ' ').collect::<String>());
            //     println!("result: {}", 0.0);
            // }
            0.0
        }
    }
}
