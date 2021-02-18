use super::{Node, ProcContext};

pub struct ProcOnce<T, A, DA>
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

impl<T, A, DA> ProcOnce<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(node: DA) -> Self {
        ProcOnce {
            node,
            time: -1.0,
            value: Default::default(),
            _a: Default::default(),
        }
    }

    pub fn as_ref(&self) -> &DA {
        &self.node
    }

    pub fn as_mut(&mut self) -> &mut DA {
        &mut self.node
    }
}

impl<T, A, DA> Node<T> for ProcOnce<T, A, DA>
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

    fn lock(&mut self) {
        self.node.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.node.as_mut().unlock();
    }
}

impl<T, A, DA> AsMut<Self> for ProcOnce<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
