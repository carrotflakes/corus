use super::{Node, ProcContext};

#[derive(Clone)]
pub struct Constant<T: Clone + 'static> {
    value: T,
}

impl<T: Clone + 'static> Constant<T> {
    pub fn new(value: T) -> Self {
        Constant {
            value,
        }
    }
}

impl<T: Clone + 'static> Node<T> for Constant<T> {
    fn proc(&mut self, _ctx: &ProcContext) -> T {
        self.value.clone()
    }
}
