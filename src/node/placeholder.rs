use std::marker::PhantomData;

use crate::proc_context::ProcContext;

use super::Node;

pub struct Placeholder<T, A, DA>
where
    T: 'static,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    node: Box<Option<DA>>,
    _t: PhantomData<T>,
    _a: PhantomData<A>,
}

pub struct PlaceholderSetter<T, A, DA>
where
    T: 'static,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    ptr: *mut Option<DA>,
    _t: PhantomData<T>,
    _a: PhantomData<A>,
}

impl<T, A, DA> Placeholder<T, A, DA>
where
    T: 'static,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(node: Option<DA>) -> Self {
        Placeholder {
            node: Box::new(node),
            _t: Default::default(),
            _a: Default::default(),
        }
    }

    pub fn set(&mut self, node: DA) {
        self.node.replace(node);
    }

    pub fn setter(&mut self) -> PlaceholderSetter<T, A, DA> {
        PlaceholderSetter {
            ptr: self.node.as_mut(),
            _t: self._t,
            _a: self._a,
        }
    }
}

impl<T, A, DA> PlaceholderSetter<T, A, DA>
where
    T: 'static,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    pub unsafe fn set(&mut self, node: DA) {
        let mut placeholder = Box::from_raw(self.ptr);
        placeholder.replace(node);
        std::mem::forget(placeholder);
    }
}

impl<T, A, DA> Node<T> for Placeholder<T, A, DA>
where
    T: 'static,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.node
            .as_mut()
            .as_mut()
            .expect("Placeholder unset")
            .as_mut()
            .proc(ctx)
    }

    fn lock(&mut self) {
        self.node
            .as_mut()
            .as_mut()
            .expect("Placeholder unset")
            .as_mut()
            .lock();
    }

    fn unlock(&mut self) {
        self.node
            .as_mut()
            .as_mut()
            .expect("Placeholder unset")
            .as_mut()
            .unlock();
    }
}

impl<T, A, DA> AsMut<Self> for Placeholder<T, A, DA>
where
    T: 'static,
    A: Node<T> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
