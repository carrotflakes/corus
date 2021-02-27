use super::{Node, ProcContext};

pub struct ProcOnce<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    node: A,
    time: f64,
    value: T,
    pub(crate) lock_count: u32,
}

impl<T, A> ProcOnce<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    pub fn new(node: A) -> Self {
        ProcOnce {
            node,
            time: -1.0,
            value: Default::default(),
            lock_count: 0,
        }
    }

    pub(crate) fn get_ref(&self) -> &A {
        &self.node
    }

    // pub(crate) fn get_mut(&mut self) -> &mut DA {
    //     &mut self.node
    // }
}

impl<T, A> Node<T> for ProcOnce<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        if self.time != ctx.time {
            self.time = ctx.time;
            self.value = self.node.proc(ctx);
        }
        self.value.clone()
    }

    fn lock(&mut self) {
        self.lock_count += 1;
        if self.lock_count == 1 {
            self.node.lock();
        }
    }

    fn unlock(&mut self) {
        self.lock_count -= 1;
        if self.lock_count == 0 {
            self.node.unlock();
        }
    }
}
