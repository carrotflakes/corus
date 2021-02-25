use crate::proc_context::ProcContext;

pub trait Node<T: 'static> {
    fn proc(&mut self, ctx: &ProcContext) -> T;
    fn lock(&mut self);
    fn unlock(&mut self);
}
