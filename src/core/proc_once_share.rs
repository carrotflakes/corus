use std::sync::Arc;

use super::{proc_once::ProcOnce, Node, ProcContext};

pub struct ProcOnceShare<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    proc_once: Arc<ProcOnce<T, A>>,
}

impl<T, A> ProcOnceShare<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    pub fn new(node: A) -> Self {
        ProcOnceShare {
            proc_once: Arc::new(ProcOnce::new(node)),
        }
    }

    pub(crate) fn get_ref(&self) -> &ProcOnce<T, A> {
        unsafe { std::mem::transmute::<_, &ProcOnce<T, A>>(Arc::as_ptr(&self.proc_once)) }
    }

    #[inline]
    fn get_mut(&mut self) -> &mut ProcOnce<T, A> {
        unsafe { std::mem::transmute::<_, &mut ProcOnce<T, A>>(Arc::as_ptr(&mut self.proc_once)) }
    }
}

impl<T, A> Node<T> for ProcOnceShare<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.get_mut().proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.get_mut().lock(ctx);
    }

    fn unlock(&mut self) {
        self.get_mut().unlock();
    }
}

impl<T, A> Clone for ProcOnceShare<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    fn clone(&self) -> Self {
        Self {
            proc_once: self.proc_once.clone(),
        }
    }
}

impl<T, A> From<Arc<ProcOnce<T, A>>> for ProcOnceShare<T, A>
where
    T: 'static + Clone + Default,
    A: Node<T>,
{
    fn from(node: Arc<ProcOnce<T, A>>) -> Self {
        ProcOnceShare { proc_once: node }
    }
}
