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

impl<T, F> Node<T> for FnProcessor<T, F>
where
    T: 'static + Clone,
    F: FnMut() -> T,
{
    #[inline]
    fn proc(&mut self, _ctx: &ProcContext) -> T {
        (self.f)()
    }

    fn lock(&mut self) {}

    fn unlock(&mut self) {}
}

impl<T, F> AsMut<Self> for FnProcessor<T, F>
where
    T: 'static + Clone,
    F: FnMut() -> T,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
