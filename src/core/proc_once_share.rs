use std::sync::Arc;

use super::{proc_once::ProcOnce, Node, ProcContext};

pub struct ProcOnceShare<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    proc_once: Arc<ProcOnce<T, A, DA>>,
}

impl<T, A, DA> ProcOnceShare<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(node: DA) -> Self {
        ProcOnceShare {
            proc_once: Arc::new(ProcOnce::new(node)),
        }
    }

    pub(crate) fn get_ref(&self) -> &ProcOnce<T, A, DA> {
        unsafe { std::mem::transmute::<_, &ProcOnce<T, A, DA>>(Arc::as_ptr(&self.proc_once)) }
    }

    fn get_mut(&mut self) -> &mut ProcOnce<T, A, DA> {
        unsafe { std::mem::transmute::<_, &mut ProcOnce<T, A, DA>>(Arc::as_ptr(&mut self.proc_once)) }
    }
}

impl<T, A, DA> Node<T> for ProcOnceShare<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.get_mut().as_mut().proc(ctx)
    }

    fn lock(&mut self) {
        self.get_mut().as_mut().lock();
    }

    fn unlock(&mut self) {
        self.get_mut().as_mut().unlock();
    }
}

impl<T, A, DA> AsMut<Self> for ProcOnceShare<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, A, DA> Clone for ProcOnceShare<T, A, DA>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn clone(&self) -> Self {
        Self {
            proc_once: self.proc_once.clone(),
        }
    }
}
