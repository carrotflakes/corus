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
