use wavetables::tree::{Tree, Value};

#[derive(Debug, Clone)]
pub struct Config {
    pub least_depth: usize,
    pub variable_num: usize,
}

impl Config {
    pub fn generate(&self, rng: &mut impl rand::Rng) -> Tree {
        Generator::new(self, rng).generate()
    }
}

pub struct Generator<'a, R: rand::Rng> {
    config: &'a Config,
    rng: &'a mut R,
}

impl<'a, R: rand::Rng> Generator<'a, R> {
    pub fn new(config: &'a Config, rng: &'a mut R) -> Self {
        Self { config, rng }
    }

    pub fn generate(&mut self) -> Tree {
        self.generate_(0)
    }

    fn generate_(&mut self, depth: usize) -> Tree {
        if depth < self.config.least_depth {
            self.generate_branch(depth)
        } else {
            if self.rng.gen_bool(0.5) {
                self.generate_branch(depth)
            } else {
                self.generate_leaf()
            }
        }
    }

    fn generate_leaf(&mut self) -> Tree {
        if self.rng.gen_bool(0.2) {
            return Tree::Sin;
        }
        match self.rng.gen_range(0..=19) {
            0 => Tree::Sin,
            1 => Tree::Negative(Box::new(Tree::Sin)),
            2 => Tree::Triangle,
            3 => Tree::Negative(Box::new(Tree::Triangle)),
            4 => Tree::ShiftedTriangle,
            5 => Tree::Negative(Box::new(Tree::ShiftedTriangle)),
            6 => Tree::Saw,
            7 => Tree::Negative(Box::new(Tree::Saw)),
            8 => Tree::ShiftedSaw,
            9 => Tree::Negative(Box::new(Tree::ShiftedSaw)),
            10 => Tree::Square,
            11 => Tree::Negative(Box::new(Tree::Square)),
            12 => Tree::Pulse(self.generate_variable(|s| s.rng.gen_range(0.0..1.0))),
            13 => Tree::Negative(Box::new(Tree::Pulse(
                self.generate_variable(|s| s.rng.gen_range(0.0..1.0)),
            ))),
            14 => Tree::Steps(self.rng.gen_range(3..16) as f64),
            15 => Tree::Negative(Box::new(Tree::Steps(self.rng.gen_range(3..16) as f64))),
            16 => Tree::Quadratic,
            17 => Tree::Negative(Box::new(Tree::Quadratic)),
            18 => Tree::Reversed(Box::new(Tree::Quadratic)),
            19 => Tree::Reversed(Box::new(Tree::Negative(Box::new(Tree::Quadratic)))),
            _ => unreachable!(),
        }
    }

    fn generate_branch(&mut self, depth: usize) -> Tree {
        match self.rng.gen_range(0..=7) {
            0 => Tree::Join(
                Box::new(self.generate_(depth + 1)),
                Box::new(self.generate_(depth + 1)),
            ),
            1 => Tree::Shift(
                self.generate_variable(|s| {
                    if s.rng.gen_bool(0.8) {
                        match s.rng.gen_range(1..=5) {
                            1 => 0.25,
                            2 => 0.5,
                            3 => 0.75,
                            4 => 1.0 / 3.0,
                            5 => 2.0 / 3.0,
                            _ => unreachable!(),
                        }
                    } else {
                        s.rng.gen_range(0.0..1.0)
                    }
                }),
                Box::new(self.generate_(depth + 1)),
            ),
            2 => Tree::Scale(
                self.generate_variable(|s| {
                    if s.rng.gen_bool(0.8) {
                        match s.rng.gen_range(1..=9) {
                            1 => 0.25,
                            2 => 0.5,
                            3 => 0.75,
                            4 => 1.0 / 3.0,
                            5 => 2.0 / 3.0,
                            6 => 1.5,
                            7 => 2.0,
                            8 => 3.0,
                            9 => 4.0,
                            _ => unreachable!(),
                        }
                    } else {
                        s.rng.gen_range(0.1..=3.0)
                    }
                }),
                Box::new(self.generate_(depth + 1)),
            ),
            3 => Tree::Blend(
                self.generate_variable(|s| {
                    if s.rng.gen_bool(0.5) {
                        0.5
                    } else {
                        s.rng.gen_range(0.0..1.0)
                    }
                }),
                Box::new(self.generate_(depth + 1)),
                Box::new(self.generate_(depth + 1)),
            ),
            4 => Tree::DynamicBlend(
                Box::new(self.generate_(depth + 1)),
                Box::new(self.generate_(depth + 1)),
                Box::new(self.generate_(depth + 1)),
            ),
            5 => Tree::Product(
                Box::new(self.generate_(depth + 1)),
                Box::new(self.generate_(depth + 1)),
            ),
            6 => Tree::Mul(
                Box::new(self.generate_(depth + 1)),
                Box::new(self.generate_(depth + 1)),
            ),
            7 => {
                let child = Box::new(self.generate_(depth + 1));
                Tree::Blend(
                    Value::Constant(0.5),
                    child.clone(),
                    Box::new(Tree::Scale(Value::Constant(2.0), child)),
                )
            }
            _ => unreachable!(),
        }
    }

    fn generate_variable(&mut self, value_fn: impl Fn(&mut Self) -> f64) -> Value {
        if self.config.variable_num > 0 && self.rng.gen_bool(0.5) {
            Value::Variable(self.rng.gen_range(0..self.config.variable_num))
        } else {
            Value::Constant(value_fn(self))
        }
    }
}
