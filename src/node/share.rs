use super::{Node, ProcContext};

pub struct Share<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    node: DA,
    time: f64,
    value: T,
    _a: std::marker::PhantomData<A>,
}

impl<T, A, DA> Share<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(node: DA) -> Self {
        Share {
            node,
            time: -1.0,
            value: Default::default(),
            _a: Default::default(),
        }
    }
}

impl<T, A, DA> Node<T> for Share<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn proc(&mut self, ctx: &ProcContext) -> T {
        if self.time != ctx.time {
            self.time = ctx.time;
            self.value = self.node.as_mut().proc(ctx);
        }
        self.value.clone()
    }
}

impl<T, A, DA> AsMut<Self> for Share<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
