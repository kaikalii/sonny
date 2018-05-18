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

impl Variable {
    pub fn to_f64(&self) -> f64 {
        use self::Variable::*;
        match *self {
            Number(ref x) => *x,
            Array(ref x) => x.into_iter()
                .next()
                .cloned()
                .unwrap_or(Variable::Number(0.0))
                .to_f64(),
        }
    }
}

impl PartialEq for Variable {
    fn eq(&self, b: &Variable) -> bool {
        use self::Variable::*;
        match self {
            Number(x) => match b {
                Number(y) => *x == *y,
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
                Array(ref y) => Array(y.iter().map(|y| Number(x) + y.clone()).collect()),
            },
            Array(ref x) => match b {
                Number(y) => Array(x.iter().map(|x| x.clone() + Number(y)).collect()),
                Array(ref y) => Array(
                    x.iter()
                        .zip(y.iter())
                        .map(|(x, y)| x.clone() + y.clone())
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
                Array(ref y) => Array(y.iter().map(|y| Number(x) - y.clone()).collect()),
            },
            Array(ref x) => match b {
                Number(y) => Array(x.iter().map(|x| x.clone() - Number(y)).collect()),
                Array(ref y) => Array(
                    x.iter()
                        .zip(y.iter())
                        .map(|(x, y)| x.clone() - y.clone())
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
                Array(ref y) => Array(y.iter().map(|y| Number(x) * y.clone()).collect()),
            },
            Array(ref x) => match b {
                Number(y) => Array(x.iter().map(|x| x.clone() * Number(y)).collect()),
                Array(ref y) => Array(
                    x.iter()
                        .zip(y.iter())
                        .map(|(x, y)| x.clone() * y.clone())
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
                Array(ref y) => Array(y.iter().map(|y| Number(x) / y.clone()).collect()),
            },
            Array(ref x) => match b {
                Number(y) => Array(x.iter().map(|x| x.clone() / Number(y)).collect()),
                Array(ref y) => Array(
                    x.iter()
                        .zip(y.iter())
                        .map(|(x, y)| x.clone() / y.clone())
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
                Array(ref y) => Array(y.iter().map(|y| Number(x) % y.clone()).collect()),
            },
            Array(ref x) => match b {
                Number(y) => Array(x.iter().map(|x| x.clone() % Number(y)).collect()),
                Array(ref y) => Array(
                    x.iter()
                        .zip(y.iter())
                        .map(|(x, y)| x.clone() % y.clone())
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
            Array(ref x) => Array(x.iter().map(|x| -x.clone()).collect()),
        }
    }
}

impl ops::Index<Variable> for Variable {
    type Output = Variable;
    fn index(&self, i: Variable) -> &Variable {
        use self::Variable::*;
        match *self {
            Number(..) => self,
            Array(ref x) => &x[i.to_f64() as usize],
        }
    }
}

impl Variable {
    pub fn pow(&self, power: Variable) -> Variable {
        use self::Variable::*;
        match *self {
            Number(x) => match power {
                Number(y) => Number(x.powf(y)),
                Array(ref y) => Array(y.iter().map(|y| Number(x).pow(y.clone())).collect()),
            },
            Array(ref x) => match power {
                Number(y) => Array(x.iter().map(|x| x.pow(Number(y))).collect()),
                Array(ref y) => Array(
                    x.iter()
                        .zip(y.iter())
                        .map(|(x, y)| x.pow(y.clone()))
                        .collect(),
                ),
            },
        }
    }
    pub fn min(&self, power: Variable) -> Variable {
        use self::Variable::*;
        match *self {
            Number(x) => match power {
                Number(y) => Number(x.min(y)),
                Array(ref y) => Array(y.iter().map(|y| Number(x).min(y.clone())).collect()),
            },
            Array(ref x) => match power {
                Number(y) => Array(x.iter().map(|x| x.min(Number(y))).collect()),
                Array(ref y) => Array(
                    x.iter()
                        .zip(y.iter())
                        .map(|(x, y)| x.min(y.clone()))
                        .collect(),
                ),
            },
        }
    }
    pub fn max(&self, power: Variable) -> Variable {
        use self::Variable::*;
        match *self {
            Number(x) => match power {
                Number(y) => Number(x.max(y)),
                Array(ref y) => Array(y.iter().map(|y| Number(x).max(y.clone())).collect()),
            },
            Array(ref x) => match power {
                Number(y) => Array(x.iter().map(|x| x.max(Number(y))).collect()),
                Array(ref y) => Array(
                    x.iter()
                        .zip(y.iter())
                        .map(|(x, y)| x.max(y.clone()))
                        .collect(),
                ),
            },
        }
    }
    pub fn ln(&self) -> Variable {
        use self::Variable::*;
        match *self {
            Number(x) => Number(x.log(f64::consts::E)),
            Array(ref x) => Array(x.iter().map(|x| x.ln()).collect()),
        }
    }
    pub fn sin(&self) -> Variable {
        use self::Variable::*;
        match *self {
            Number(x) => Number(x.sin()),
            Array(ref x) => Array(x.iter().map(|x| x.sin()).collect()),
        }
    }
    pub fn cos(&self) -> Variable {
        use self::Variable::*;
        match *self {
            Number(x) => Number(x.cos()),
            Array(ref x) => Array(x.iter().map(|x| x.cos()).collect()),
        }
    }
    pub fn floor(&self) -> Variable {
        use self::Variable::*;
        match *self {
            Number(x) => Number(x.floor()),
            Array(ref x) => Array(x.iter().map(|x| x.floor()).collect()),
        }
    }
    pub fn ceil(&self) -> Variable {
        use self::Variable::*;
        match *self {
            Number(x) => Number(x.ceil()),
            Array(ref x) => Array(x.iter().map(|x| x.ceil()).collect()),
        }
    }
    pub fn abs(&self) -> Variable {
        use self::Variable::*;
        match *self {
            Number(x) => Number(x.abs()),
            Array(ref x) => Array(x.iter().map(|x| x.abs()).collect()),
        }
    }
    pub fn sub_array(&self, start: &Variable, end: &Variable) -> Variable {
        use self::Variable::*;
        match *self {
            Number(..) => self.clone(),
            Array(ref x) => Array(
                x.iter()
                    .skip(start.to_f64() as usize)
                    .take((end.clone() - start.clone()).to_f64() as usize)
                    .cloned()
                    .collect(),
            ),
        }
    }
    pub fn average(&self) -> Variable {
        use self::Variable::*;
        match *self {
            Number(..) => self.clone(),
            Array(ref x) => {
                x.iter().fold(Number(0.0), |sum, val| sum + val.clone()) / Number(x.len() as f64)
            }
        }
    }
}
