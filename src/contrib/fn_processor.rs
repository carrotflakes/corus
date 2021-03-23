use crate::proc_context::ProcContext;

use super::Node;

pub struct FnProcessor<T, F>
where
    T: 'static + Clone,
    F: FnMut() -> T,
{
    f: F,
}

impl<T, F> FnProcessor<T, F>
where
    T: 'static + Clone,
    F: FnMut() -> T,
{
    pub fn new(f: F) -> Self {
        FnProcessor { f }
    }
}

impl<T, F> Node for FnProcessor<T, F>
where
    T: 'static + Clone,
    F: FnMut() -> T,
{
    type Output = T;

    #[inline]
    fn proc(&mut self, _ctx: &ProcContext) -> T {
        (self.f)()
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
}
