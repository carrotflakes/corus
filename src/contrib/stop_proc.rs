use std::marker::PhantomData;

use crate::Node;

pub struct StopProc<T: 'static + Default, A: Node<T>> {
    node: A,
    pub enabled: bool,
    _t: PhantomData<T>,
}

impl<T: 'static + Default, A: Node<T>> StopProc<T, A> {
    pub fn new(node: A, enabled: bool) -> Self {
        Self {
            node,
            enabled,
            _t: Default::default(),
        }
    }
}

impl<T: 'static + Default, A: Node<T>> Node<T> for StopProc<T, A> {
    fn proc(&mut self, ctx: &crate::ProcContext) -> T {
        if self.enabled {
            self.node.proc(ctx)
        } else {
            Default::default()
        }
    }

    fn lock(&mut self, ctx: &crate::ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
