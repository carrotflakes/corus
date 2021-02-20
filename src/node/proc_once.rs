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
    pub(crate) lock_count: u32,
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
            lock_count: 0,
            _a: Default::default(),
        }
    }

    pub(crate) fn get_ref(&self) -> &DA {
        &self.node
    }

    // pub(crate) fn get_mut(&mut self) -> &mut DA {
    //     &mut self.node
    // }
}

impl<T, A, DA> Node<T> for ProcOnce<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        if self.time != ctx.time {
            self.time = ctx.time;
            self.value = self.node.as_mut().proc(ctx);
        }
        self.value.clone()
    }

    fn lock(&mut self) {
        self.lock_count += 1;
        if self.lock_count == 1 {
            self.node.as_mut().lock();
        }
    }

    fn unlock(&mut self) {
        self.lock_count -= 1;
        if self.lock_count == 0 {
            self.node.as_mut().unlock();
        }
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
