use super::{Node, ProcContext};

pub struct Map<T, S, F, A>
where
    T: Clone + 'static,
    S: Clone + 'static,
    F: Fn(T) -> S,
    A: Node<T>,
{
    node: A,
    f: F,
    _t: std::marker::PhantomData<T>,
}

impl<T, S, F, A> Map<T, S, F, A>
where
    T: Clone + 'static,
    S: Clone + 'static,
    F: Fn(T) -> S,
    A: Node<T>,
{
    pub fn new(node: A, f: F) -> Self {
        Map {
            node,
            f,
            _t: Default::default(),
        }
    }
}

impl<T, S, F, A> Node<S> for Map<T, S, F, A>
where
    T: Clone + 'static,
    S: Clone + 'static,
    F: Fn(T) -> S,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> S {
        (self.f)(self.node.proc(ctx))
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
