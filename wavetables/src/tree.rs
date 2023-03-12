use crate::{functions, primitives};

#[derive(Clone)]
pub enum Tree {
    Sin,
    Triangle,
    ShiftedTriangle,
    Saw,
    Square,
    Pulse(f64),
    Quadratic,

    Negative(Box<Tree>),
    Reversed(Box<Tree>),
    Join(Box<Tree>, Box<Tree>),
    Shift(f64, Box<Tree>),
    Scale(f64, Box<Tree>),
    Blend(f64, Box<Tree>, Box<Tree>),
    DynamicBlend(Box<Tree>, Box<Tree>, Box<Tree>),
    Dot(Box<Tree>, Box<Tree>),
}

impl Tree {
    pub fn build(&self) -> Box<dyn Fn(f64) -> f64> {
        match self {
            Tree::Sin => Box::new(primitives::sin),
            Tree::Triangle => Box::new(primitives::triangle),
            Tree::ShiftedTriangle => Box::new(primitives::shifted_triangle),
            Tree::Saw => Box::new(primitives::saw),
            Tree::Square => Box::new(primitives::square),
            Tree::Pulse(width) => {
                let width = *width;
                Box::new(move |t| primitives::pulse(width, t))
            }
            Tree::Quadratic => Box::new(primitives::quadratic),
            Tree::Negative(f) => Box::new(functions::negative(f.build())),
            Tree::Reversed(f) => Box::new(functions::reversed(f.build())),
            Tree::Join(f1, f2) => Box::new(functions::join(f1.build(), f2.build())),
            Tree::Shift(shift, f) => Box::new(functions::shift(*shift, f.build())),
            Tree::Scale(scale, f) => Box::new(functions::scale(*scale, f.build())),
            Tree::Blend(t, f1, f2) => Box::new(functions::blend(*t, f1.build(), f2.build())),
            Tree::DynamicBlend(f, f1, f2) => {
                Box::new(functions::dynamic_blend(f.build(), f1.build(), f2.build()))
            }
            Tree::Dot(f1, f2) => Box::new(functions::dot(f1.build(), f2.build())),
        }
    }
}
