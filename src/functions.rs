use std::collections::{HashMap, HashSet};

use builder::*;

#[derive(Debug, Clone)]
struct FunctionDef {
    pub true_args: Vec<HashSet<usize>>,
    pub chain: Chain,
}

#[derive(Debug)]
pub struct Functions {
    builder: Builder,
    functions: HashMap<ChainName, FunctionDef>,
}

impl Functions {
    pub fn new(builder: Builder) -> Functions {
        let mut f = Functions {
            builder,
            functions: HashMap::new(),
        };
        f.make_functions();
        f
    }
    fn collect_args(&mut self, expression: &Expression) -> HashSet<usize> {
        let mut args = HashSet::new();
        let operands = expression.operation.operands();
        use Operand::*;
        match *operands.0 {
            Id(ref id) => {
                if !self.functions.contains_key(&ChainName::String(id.clone())) {
                    let chain = self.builder.chains[&ChainName::String(id.clone())].clone();
                    self.make_function(chain);
                }
                args = args.union(
                    self.functions[&ChainName::String(id.clone())]
                        .true_args
                        .last()
                        .unwrap(),
                ).cloned()
                    .collect();
            }
            Expression(ref expr) => {
                args = args.union(&self.collect_args(&expr)).cloned().collect();
            }
            _ => (),
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
            self.make_function(chain);
        }
    }
}
