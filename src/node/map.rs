use super::{Node, ProcContext};

pub struct Map<T, F, A, DA>
where
    T: Clone + 'static,
    F: Fn(T) -> T,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    node: DA,
    f: F,
    _t: std::marker::PhantomData<T>,
    _a: std::marker::PhantomData<A>,
}

impl<T, F, A, DA> Map<T, F, A, DA>
where
    T: Clone + 'static,
    F: Fn(T) -> T,
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

impl<T, F, A, DA> Node<T> for Map<T, F, A, DA>
where
    T: Clone + 'static,
    F: Fn(T) -> T,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn proc(&mut self, ctx: &ProcContext) -> T {
        (self.f)(self.node.as_mut().proc(ctx))
    }

    fn lock(&mut self) {
        self.node.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.node.as_mut().unlock();
    }
}

impl<T, F, A, DA> AsMut<Self> for Map<T, F, A, DA>
where
    T: Clone + 'static,
    F: Fn(T) -> T,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
