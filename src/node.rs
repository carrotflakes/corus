use std::sync::{Arc, Mutex};

use crate::proc_context::ProcContext;

pub trait Node<T: 'static> {
    fn proc(&mut self, ctx: &ProcContext) -> T;
    fn lock(&mut self, ctx: &ProcContext);
    fn unlock(&mut self);
}

impl<T: 'static, N: Node<T> + ?Sized> Node<T> for Box<N> {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.as_mut().proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.as_mut().lock(ctx);
    }

    fn unlock(&mut self) {
        self.as_mut().unlock();
    }
}

impl<T, A> Node<T> for Arc<A>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        get_mut(self).proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        if Arc::strong_count(self) != 1 {
            panic!("Cloned Arc<Node<_>> cannot be proc!");
        }
        get_mut(self).lock(ctx);
    }

    fn unlock(&mut self) {
        get_mut(self).unlock();
    }
}

#[inline]
fn get_mut<T, A>(arc: &mut Arc<A>) -> &mut A
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
{
    unsafe { std::mem::transmute::<_, &mut A>(Arc::as_ptr(arc)) }
}


impl<T, A> Node<T> for Mutex<A>
where
    T: 'static + Clone + Default,
    A: Node<T> + ?Sized,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.get_mut().unwrap().proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.get_mut().unwrap().lock(ctx);
    }

    fn unlock(&mut self) {
        self.get_mut().unwrap().unlock();
    }
}
