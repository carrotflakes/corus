use std::ops::Add;

use super::{Node, ProcContext};

pub struct Mix<T, A>
where
    T: Clone + 'static + Add<Output = T> + Default,
    A: Node<T>,
{
    nodes: Vec<A>,
    _t: std::marker::PhantomData<T>,
}

impl<T, A> Mix<T, A>
where
    T: Clone + 'static + Add<Output = T> + Default,
    A: Node<T>,
{
    pub fn new(nodes: Vec<A>) -> Self {
        Mix {
            nodes,
            _t: Default::default(),
        }
    }
}

impl<T, A> Node<T> for Mix<T, A>
where
    T: Clone + 'static + Add<Output = T> + Default,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let mut v = Default::default();
        for node in self.nodes.iter_mut() {
            v = v + node.proc(ctx);
        }
        v
    }

    fn lock(&mut self, ctx: &ProcContext) {
        for node in &mut self.nodes {
            node.lock(ctx);
        }
    }

    fn unlock(&mut self) {
        for node in &mut self.nodes {
            node.unlock();
        }
    }
}
