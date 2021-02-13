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
    fn procedure(&self) -> Box<dyn FnMut(&ProcContext) -> T> {
        let mut a = self.a.procedure();
        let mut b = self.b.procedure();
        Box::new(move |ctx| {
            a(ctx) + b(ctx)
        })
    }
}
