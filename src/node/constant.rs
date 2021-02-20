use super::{Node, ProcContext};

#[derive(Clone)]
pub struct Constant<T: Clone + 'static> {
    value: T,
}

impl<T: Clone + 'static> Constant<T> {
    pub fn new(value: T) -> Self {
        Constant { value }
    }

    pub fn from<S: Clone + 'static + Into<T>>(src: S) -> Self {
        Constant { value: src.into() }
    }
}

impl<T: Clone + 'static> Node<T> for Constant<T> {
    #[inline]
    fn proc(&mut self, _ctx: &ProcContext) -> T {
        self.value.clone()
    }

    fn lock(&mut self) {}

    fn unlock(&mut self) {}
}

impl<T: Clone + 'static> AsMut<Constant<T>> for Constant<T> {
    fn as_mut(&mut self) -> &mut Constant<T> {
        self
    }
}
