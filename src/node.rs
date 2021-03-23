use crate::proc_context::ProcContext;

pub trait Node {
    type Output: 'static;

    fn proc(&mut self, ctx: &ProcContext) -> Self::Output;
    fn lock(&mut self, ctx: &ProcContext);
    fn unlock(&mut self);
}

impl<N: Node + ?Sized> Node for Box<N> {
    type Output = N::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        self.as_mut().proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.as_mut().lock(ctx);
    }

    fn unlock(&mut self) {
        self.as_mut().unlock();
    }
}
