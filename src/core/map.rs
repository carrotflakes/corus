use super::{Node, ProcContext};

pub struct Map<T, S, F, A, DA>
where
    T: Clone + 'static,
    S: Clone + 'static,
    F: Fn(T) -> S,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    node: DA,
    f: F,
    _t: std::marker::PhantomData<T>,
    _a: std::marker::PhantomData<A>,
}

impl<T, S, F, A, DA> Map<T, S, F, A, DA>
where
    T: Clone + 'static,
    S: Clone + 'static,
    F: Fn(T) -> S,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(node: DA, f: F) -> Self {
        Map {
            node,
            f,
            _t: Default::default(),
            _a: Default::default(),
        }
    }
}

impl<T, S, F, A, DA> Node<S> for Map<T, S, F, A, DA>
where
    T: Clone + 'static,
    S: Clone + 'static,
    F: Fn(T) -> S,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> S {
        (self.f)(self.node.as_mut().proc(ctx))
    }

    fn lock(&mut self) {
        self.node.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.node.as_mut().unlock();
    }
}

impl<T, S, F, A, DA> AsMut<Self> for Map<T, S, F, A, DA>
where
    T: Clone + 'static,
    S: Clone + 'static,
    F: Fn(T) -> S,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
