use std::collections::{HashMap, HashSet};
use std::f64;

use builder::*;

impl Builder {
    fn collect_args(
        &self,
        expression: &Expression,
        true_args_map: &mut HashMap<ChainName, Vec<HashSet<usize>>>,
    ) -> HashSet<usize> {
        let mut args = HashSet::new();
        let operands = expression.0.operands();
        use Operand::*;
        match *operands.0 {
            Id(ref id) => {
                if self.find_chain(id).is_none() {
                    self.make_function(
                        &self.find_chain(id).expect("Unable to find chain").name,
                        true_args_map,
                    );
                }
                if !self.find_chain(id).unwrap().true_args.is_empty() {
                    args = args.union(self.find_chain(id).unwrap().true_args.last().unwrap())
                        .cloned()
                        .collect();
                }
            }
            Expression(ref expr) => {
                args = args.union(&self.collect_args(&expr, true_args_map))
                    .cloned()
                    .collect();
            }
            BackLink(num) => {
                args.insert(num);
            }
            _ => (),
        }
        if let Some(op) = operands.1 {
            match *op {
                Id(ref id) => {
                    if self.find_chain(id).is_none() {
                        self.make_function(
                            &self.find_chain(id).expect("Unable to find chain").name,
                            true_args_map,
                        );
                    }
                    if !self.find_chain(id).unwrap().true_args.is_empty() {
                        args = args.union(self.find_chain(id).unwrap().true_args.last().unwrap())
                            .cloned()
                            .collect();
                    }
                }
                Expression(ref expr) => {
                    args = args.union(&self.collect_args(&expr, true_args_map))
                        .cloned()
                        .collect();
                }
                BackLink(num) => {
                    args.insert(num);
                }
                _ => (),
            }
        }
        args
    }

    fn make_function(
        &self,
        name: &ChainName,
        true_args_map: &mut HashMap<ChainName, Vec<HashSet<usize>>>,
    ) {
        let mut true_args: Vec<HashSet<usize>> = Vec::new();
        if let ChainLinks::Generic(ref expressions) =
            self.find_chain(name).expect("Unable to find chain").links
        {
            for (i, expression) in expressions.iter().enumerate() {
                let temp_args = self.collect_args(expression, true_args_map);
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
        true_args_map.insert(name.clone(), true_args);
    }

    pub fn make_functions(&mut self) {
        let mut true_args_map = HashMap::new();
        for name in self.chains
            .values()
            .filter(|c| c.play)
            .map(|c| c.name.clone())
            .collect::<Vec<ChainName>>()
        {
            if self.find_chain(&name).is_none() {
                self.make_function(&name, &mut true_args_map);
            }
        }
        for (name, true_args) in true_args_map {
            self.chains
                .get_mut(&name)
                .expect("Unable to find chain")
                .true_args = true_args;
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
            Property(ref id, property) => if let Some(chain) = self.find_chain(id) {
                if let ChainLinks::OnlyNotes(..) = chain.links {
                    if let Some(note) = chain.links.find_note(time, 0.0, &self) {
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
        if let Some(chain) = self.find_chain(name) {
            match chain.links {
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
                ChainLinks::OnlyNotes(..) => chain
                    .links
                    .find_note(time, 0.0, &self)
                    .map(|n| n.pitch)
                    .unwrap_or(0.0),
            }
        } else {
            panic!("No function named '{}'", name);
        }
    }
}