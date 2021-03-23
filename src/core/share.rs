use std::sync::Arc;

use super::{proc_once::ProcOnce, Node, ProcContext};

pub struct Share<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    proc_once: Arc<ProcOnce<A>>,
}

impl<A> Share<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    pub fn new(node: A) -> Self {
        Share {
            proc_once: Arc::new(ProcOnce::new(node)),
        }
    }

    pub(crate) fn get_ref(&self) -> &ProcOnce<A> {
        unsafe { std::mem::transmute::<_, &ProcOnce<A>>(Arc::as_ptr(&self.proc_once)) }
    }

    #[inline]
    fn get_mut(&mut self) -> &mut ProcOnce<A> {
        unsafe { std::mem::transmute::<_, &mut ProcOnce<A>>(Arc::as_ptr(&mut self.proc_once)) }
    }
}

impl<A> Node for Share<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        self.get_mut().proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.get_mut().lock(ctx);
    }

    fn unlock(&mut self) {
        self.get_mut().unlock();
    }
}

impl<A> Clone for Share<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    fn clone(&self) -> Self {
        Self {
            proc_once: self.proc_once.clone(),
        }
    }
}

impl<A> From<Arc<ProcOnce<A>>> for Share<A>
where
    A: Node,
    A::Output: Clone + Default,
{
    fn from(node: Arc<ProcOnce<A>>) -> Self {
        Share { proc_once: node }
    }
}
