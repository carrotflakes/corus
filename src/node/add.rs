use super::{Node, ProcContext};

pub struct Add<T: Clone + 'static + std::ops::Add<Output=T>> {
    a: Box<dyn Node<T>>,
    b: Box<dyn Node<T>>,
}

impl<T: Clone + 'static + std::ops::Add<Output=T>> Add<T> {
    pub fn new(a: Box<dyn Node<T>>, b: Box<dyn Node<T>>) -> Self {
        Add {
            a,
            b
        }
    }
}

impl<T: Clone + 'static + std::ops::Add<Output=T>> Node<T> for Add<T> {
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.a.proc(ctx) + self.b.proc(ctx)
    }
}
