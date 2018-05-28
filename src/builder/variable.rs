use std::cmp::Ordering;
use std::f64;
use std::ops;

// A universal variable type
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Variable {
    Number(f64),
    Array(Vec<Variable>),
}

impl From<Variable> for f64 {
    fn from(v: Variable) -> f64 {
        use self::Variable::*;
        match v {
            Number(x) => x,
            Array(x) => x.into_iter().next().unwrap_or(Variable::Number(0.0)).into(),
        }
    }
}

impl<'a> From<&'a str> for Variable {
    fn from(s: &str) -> Variable {
        Variable::Array(
            s.chars()
                .map(|c| Variable::Number(c as u32 as f64))
                .collect(),
        )
    }
}

impl PartialEq for Variable {
    fn eq(&self, b: &Variable) -> bool {
        use self::Variable::*;
        match self {
            Number(x) => match *b {
                Number(y) => *x == y,
                Array(ref y) => y.iter().all(|y| Number(*x) == *y),
            },
            Array(ref x) => match b {
                Number(y) => x.iter().all(|x| *x == Number(*y)),
                Array(ref y) => x.iter().zip(y.iter()).all(|(x, y)| x == y),
            },
        }
    }
}

impl PartialOrd for Variable {
    fn partial_cmp(&self, b: &Variable) -> Option<Ordering> {
        use self::Variable::*;
        match self {
            Number(x) => match b {
                Number(y) => x.partial_cmp(y),
                Array(..) => None,
            },
            Array(..) => match b {
                Number(..) => None,
                Array(..) => None,
            },
        }
    }
}

impl ops::Add<Variable> for Variable {
    type Output = Variable;
    fn add(self, b: Variable) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => match b {
                Number(y) => Number(x + y),
                Array(y) => Array(y.into_iter().map(|y| Number(x) + y).collect()),
            },
            Array(x) => match b {
                Number(y) => Array(x.into_iter().map(|x| x + Number(y)).collect()),
                Array(y) => Array(
                    x.into_iter()
                        .zip(y.into_iter())
                        .map(|(x, y)| x + y)
                        .collect(),
                ),
            },
        }
    }
}

impl ops::Sub<Variable> for Variable {
    type Output = Variable;
    fn sub(self, b: Variable) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => match b {
                Number(y) => Number(x - y),
                Array(y) => Array(y.into_iter().map(|y| Number(x) - y).collect()),
            },
            Array(x) => match b {
                Number(y) => Array(x.into_iter().map(|x| x - Number(y)).collect()),
                Array(y) => Array(
                    x.into_iter()
                        .zip(y.into_iter())
                        .map(|(x, y)| x - y)
                        .collect(),
                ),
            },
        }
    }
}

impl ops::Mul<Variable> for Variable {
    type Output = Variable;
    fn mul(self, b: Variable) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => match b {
                Number(y) => Number(x * y),
                Array(y) => Array(y.into_iter().map(|y| Number(x) * y).collect()),
            },
            Array(x) => match b {
                Number(y) => Array(x.into_iter().map(|x| x * Number(y)).collect()),
                Array(y) => Array(
                    x.into_iter()
                        .zip(y.into_iter())
                        .map(|(x, y)| x * y)
                        .collect(),
                ),
            },
        }
    }
}

impl ops::Div<Variable> for Variable {
    type Output = Variable;
    fn div(self, b: Variable) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => match b {
                Number(y) => Number(x / y),
                Array(y) => Array(y.into_iter().map(|y| Number(x) / y).collect()),
            },
            Array(x) => match b {
                Number(y) => Array(x.into_iter().map(|x| x / Number(y)).collect()),
                Array(y) => Array(
                    x.into_iter()
                        .zip(y.into_iter())
                        .map(|(x, y)| x / y)
                        .collect(),
                ),
            },
        }
    }
}

impl ops::Rem<Variable> for Variable {
    type Output = Variable;
    fn rem(self, b: Variable) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => match b {
                Number(y) => Number(x % y),
                Array(y) => Array(y.into_iter().map(|y| Number(x) % y).collect()),
            },
            Array(x) => match b {
                Number(y) => Array(x.into_iter().map(|x| x % Number(y)).collect()),
                Array(y) => Array(
                    x.into_iter()
                        .zip(y.into_iter())
                        .map(|(x, y)| x % y)
                        .collect(),
                ),
            },
        }
    }
}

impl ops::Neg for Variable {
    type Output = Variable;
    fn neg(self) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => Number(-x),
            Array(x) => Array(x.into_iter().map(|x| -x).collect()),
        }
    }
}

impl ops::Index<Variable> for Variable {
    type Output = Variable;
    fn index(&self, i: Variable) -> &Variable {
        use self::Variable::*;
        match *self {
            Number(..) => self,
            Array(ref x) => &x[f64::from(i) as usize],
        }
    }
}

impl Variable {
    pub fn pow(self, power: Variable) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => match power {
                Number(y) => Number(x.powf(y)),
                Array(y) => Array(y.into_iter().map(|y| Number(x).pow(y)).collect()),
            },
            Array(x) => match power {
                Number(y) => Array(x.into_iter().map(|x| x.pow(Number(y))).collect()),
                Array(y) => Array(
                    x.into_iter()
                        .zip(y.into_iter())
                        .map(|(x, y)| x.pow(y))
                        .collect(),
                ),
            },
        }
    }
    pub fn min(self, power: Variable) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => match power {
                Number(y) => Number(x.min(y)),
                Array(y) => Array(y.into_iter().map(|y| Number(x).min(y)).collect()),
            },
            Array(x) => match power {
                Number(y) => Array(x.into_iter().map(|x| x.min(Number(y))).collect()),
                Array(y) => Array(
                    x.into_iter()
                        .zip(y.into_iter())
                        .map(|(x, y)| x.min(y))
                        .collect(),
                ),
            },
        }
    }
    pub fn max(self, power: Variable) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => match power {
                Number(y) => Number(x.max(y)),
                Array(y) => Array(y.into_iter().map(|y| Number(x).max(y)).collect()),
            },
            Array(x) => match power {
                Number(y) => Array(x.into_iter().map(|x| x.max(Number(y))).collect()),
                Array(y) => Array(
                    x.into_iter()
                        .zip(y.into_iter())
                        .map(|(x, y)| x.max(y))
                        .collect(),
                ),
            },
        }
    }
    pub fn ln(self) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => Number(x.log(f64::consts::E)),
            Array(x) => Array(x.into_iter().map(|x| x.ln()).collect()),
        }
    }
    pub fn sin(self) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => Number(x.sin()),
            Array(x) => Array(x.into_iter().map(|x| x.sin()).collect()),
        }
    }
    pub fn cos(self) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => Number(x.cos()),
            Array(x) => Array(x.into_iter().map(|x| x.cos()).collect()),
        }
    }
    pub fn floor(self) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => Number(x.floor()),
            Array(x) => Array(x.into_iter().map(|x| x.floor()).collect()),
        }
    }
    pub fn ceil(self) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => Number(x.ceil()),
            Array(x) => Array(x.into_iter().map(|x| x.ceil()).collect()),
        }
    }
    pub fn abs(self) -> Variable {
        use self::Variable::*;
        match self {
            Number(x) => Number(x.abs()),
            Array(x) => Array(x.into_iter().map(|x| x.abs()).collect()),
        }
    }
    pub fn sub_array(self, start: Variable, end: Variable) -> Variable {
        use self::Variable::*;
        match self {
            Number(..) => self,
            Array(x) => Array(
                x.into_iter()
                    .skip(f64::from(start.clone()) as usize)
                    .take(f64::from(end.clone() - start.clone()) as usize)
                    .collect(),
            ),
        }
    }
    pub fn average(self) -> Variable {
        use self::Variable::*;
        match self {
            Number(..) => self,
            Array(x) => {
                let xlen = x.len();
                x.into_iter().fold(Number(0.0), |sum, val| sum + val) / Number(xlen as f64)
            }
        }
    }
}
