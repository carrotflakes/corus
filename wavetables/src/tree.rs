use crate::{functions, primitives};

#[derive(Debug, Clone)]
pub enum Tree {
    Sin,
    Triangle,
    ShiftedTriangle,
    Saw,
    ShiftedSaw,
    Square,
    Pulse(Value),
    Steps(f64),
    Quadratic,

    Negative(Box<Tree>),
    Reversed(Box<Tree>),
    Join(Box<Tree>, Box<Tree>),
    Shift(Value, Box<Tree>),
    Scale(Value, Box<Tree>),
    Blend(Value, Box<Tree>, Box<Tree>),
    DynamicBlend(Box<Tree>, Box<Tree>, Box<Tree>),
    Product(Box<Tree>, Box<Tree>),
    Mul(Box<Tree>, Box<Tree>),
    Mirror(Box<Tree>),
}

#[derive(Debug, Clone)]
pub enum Value {
    Constant(f64),
    Variable(usize),
}

impl Value {
    pub fn get_no_param(&self) -> f64 {
        match self {
            Value::Constant(x) => *x,
            Value::Variable(_) => panic!("tried to get a variable value without parameters"),
        }
    }

    pub fn get(&self, params: &[f64]) -> f64 {
        match self {
            Value::Constant(x) => *x,
            Value::Variable(i) => params[*i],
        }
    }
}

impl Tree {
    pub fn instant_params(&self, params: &[f64]) -> Self {
        match self {
            Tree::Sin => Tree::Sin,
            Tree::Triangle => Tree::Triangle,
            Tree::ShiftedTriangle => Tree::ShiftedTriangle,
            Tree::Saw => Tree::Saw,
            Tree::ShiftedSaw => Tree::ShiftedSaw,
            Tree::Square => Tree::Square,
            Tree::Pulse(width) => Tree::Pulse(Value::Constant(width.get(params))),
            Tree::Steps(n) => Tree::Steps(*n),
            Tree::Quadratic => Tree::Quadratic,
            Tree::Negative(f) => Tree::Negative(Box::new(f.instant_params(params))),
            Tree::Reversed(f) => Tree::Reversed(Box::new(f.instant_params(params))),
            Tree::Join(f1, f2) => Tree::Join(
                Box::new(f1.instant_params(params)),
                Box::new(f2.instant_params(params)),
            ),
            Tree::Shift(shift, f) => Tree::Shift(
                Value::Constant(shift.get(params)),
                Box::new(f.instant_params(params)),
            ),
            Tree::Scale(scale, f) => Tree::Scale(
                Value::Constant(scale.get(params)),
                Box::new(f.instant_params(params)),
            ),
            Tree::Blend(r, f1, f2) => Tree::Blend(
                Value::Constant(r.get(params)),
                Box::new(f1.instant_params(params)),
                Box::new(f2.instant_params(params)),
            ),
            Tree::DynamicBlend(f1, f2, f3) => Tree::DynamicBlend(
                Box::new(f1.instant_params(params)),
                Box::new(f2.instant_params(params)),
                Box::new(f3.instant_params(params)),
            ),
            Tree::Product(f1, f2) => Tree::Product(
                Box::new(f1.instant_params(params)),
                Box::new(f2.instant_params(params)),
            ),
            Tree::Mul(f1, f2) => Tree::Mul(
                Box::new(f1.instant_params(params)),
                Box::new(f2.instant_params(params)),
            ),
            Tree::Mirror(f) => Tree::Mirror(Box::new(f.instant_params(params))),
        }
    }

    pub fn build(&self) -> Box<dyn Fn(f64) -> f64 + Send + Sync + 'static> {
        match self {
            Tree::Sin => Box::new(primitives::sin),
            Tree::Triangle => Box::new(primitives::triangle),
            Tree::ShiftedTriangle => Box::new(primitives::shifted_triangle),
            Tree::Saw => Box::new(primitives::saw),
            Tree::ShiftedSaw => Box::new(primitives::shifted_saw),
            Tree::Square => Box::new(primitives::square),
            Tree::Pulse(width) => {
                let width = width.get_no_param();
                Box::new(move |t| primitives::pulse(width, t))
            }
            Tree::Steps(n) => {
                let n = *n;
                Box::new(move |t| primitives::steps(n, t))
            }
            Tree::Quadratic => Box::new(primitives::quadratic),
            Tree::Negative(f) => Box::new(functions::negative(f.build())),
            Tree::Reversed(f) => Box::new(functions::reversed(f.build())),
            Tree::Join(f1, f2) => Box::new(functions::join(f1.build(), f2.build())),
            Tree::Shift(shift, f) => Box::new(functions::shift(shift.get_no_param(), f.build())),
            Tree::Scale(scale, f) => Box::new(functions::scale(scale.get_no_param(), f.build())),
            Tree::Blend(r, f1, f2) => {
                Box::new(functions::blend(r.get_no_param(), f1.build(), f2.build()))
            }
            Tree::DynamicBlend(f, f1, f2) => {
                Box::new(functions::dynamic_blend(f.build(), f1.build(), f2.build()))
            }
            Tree::Product(f1, f2) => Box::new(functions::product(f1.build(), f2.build())),
            Tree::Mul(f1, f2) => Box::new(functions::mul(f1.build(), f2.build())),
            Tree::Mirror(f) => Box::new(functions::mirror(f.build())),
        }
    }

    pub fn build_parameterized(&self) -> Box<dyn Fn(&[f64], f64) -> f64 + Send + Sync + 'static> {
        match self {
            Tree::Sin => Box::new(|_params, t| primitives::sin(t)),
            Tree::Triangle => Box::new(|_params, t| primitives::triangle(t)),
            Tree::ShiftedTriangle => Box::new(|_params, t| primitives::shifted_triangle(t)),
            Tree::Saw => Box::new(|_params, t| primitives::saw(t)),
            Tree::ShiftedSaw => Box::new(|_params, t| primitives::shifted_saw(t)),
            Tree::Square => Box::new(|_params, t| primitives::square(t)),
            Tree::Pulse(width) => {
                let width = width.clone();
                Box::new(move |params, t| primitives::pulse(width.get(params), t))
            }
            Tree::Steps(n) => {
                let n = *n;
                Box::new(move |_params, t| primitives::steps(n, t))
            }
            Tree::Quadratic => Box::new(|_params, t| primitives::quadratic(t)),
            Tree::Negative(f) => {
                let f = f.build_parameterized();
                Box::new(move |params, t| functions::negative(|t| f(params, t))(t))
            }
            Tree::Reversed(f) => {
                let f = f.build_parameterized();
                Box::new(move |params, t| functions::reversed(|t| f(params, t))(t))
            }
            Tree::Join(f1, f2) => {
                let f1 = f1.build_parameterized();
                let f2 = f2.build_parameterized();
                Box::new(move |params, t| functions::join(|t| f1(params, t), |t| f2(params, t))(t))
            }
            Tree::Shift(shift, f) => {
                let shift = shift.clone();
                let f = f.build_parameterized();
                Box::new(move |params, t| functions::shift(shift.get(params), |t| f(params, t))(t))
            }
            Tree::Scale(scale, f) => {
                let scale = scale.clone();
                let f = f.build_parameterized();
                Box::new(move |params, t| functions::scale(scale.get(params), |t| f(params, t))(t))
            }
            Tree::Blend(r, f1, f2) => {
                let r = r.clone();
                let f1 = f1.build_parameterized();
                let f2 = f2.build_parameterized();
                Box::new(move |params, t| {
                    functions::blend(r.get(params), |t| f1(params, t), |t| f2(params, t))(t)
                })
            }
            Tree::DynamicBlend(f, f1, f2) => {
                let f = f.build_parameterized();
                let f1 = f1.build_parameterized();
                let f2 = f2.build_parameterized();
                Box::new(move |params, t| {
                    functions::dynamic_blend(|t| f(params, t), |t| f1(params, t), |t| f2(params, t))(
                        t,
                    )
                })
            }
            Tree::Product(f1, f2) => {
                let f1 = f1.build_parameterized();
                let f2 = f2.build_parameterized();
                Box::new(move |params, t| {
                    functions::product(|t| f1(params, t), |t| f2(params, t))(t)
                })
            }
            Tree::Mul(f1, f2) => {
                let f1 = f1.build_parameterized();
                let f2 = f2.build_parameterized();
                Box::new(move |params, t| functions::mul(|t| f1(params, t), |t| f2(params, t))(t))
            }
            Tree::Mirror(f) => {
                let f = f.build_parameterized();
                Box::new(move |params, t| functions::mirror(|t| f(params, t))(t))
            }
        }
    }
}
