use crate::tree::{Tree, Value};

pub fn unique_negative_reverse_primitives() -> Vec<Tree> {
    vec![
        Tree::Sin,
        Tree::Negative(Box::new(Tree::Sin)),
        Tree::Triangle,
        Tree::Negative(Box::new(Tree::Triangle)),
        Tree::ShiftedTriangle,
        Tree::Negative(Box::new(Tree::ShiftedTriangle)),
        Tree::Saw,
        Tree::Negative(Box::new(Tree::Saw)),
        Tree::ShiftedSaw,
        Tree::Negative(Box::new(Tree::ShiftedSaw)),
        Tree::Square,
        Tree::Negative(Box::new(Tree::Square)),
        Tree::Pulse(Value::Variable(0)),
        Tree::Negative(Box::new(Tree::Pulse(Value::Variable(0)))),
        Tree::Steps(3.0),
        Tree::Negative(Box::new(Tree::Steps(3.0))),
        Tree::Quadratic,
        Tree::Negative(Box::new(Tree::Quadratic)),
        Tree::Reversed(Box::new(Tree::Quadratic)),
        Tree::Reversed(Box::new(Tree::Negative(Box::new(Tree::Quadratic)))),
    ]
}
