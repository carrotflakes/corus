use std::marker::PhantomData;

use crate::proc_context::ProcContext;

use super::Node;

pub struct Placeholder<T, A>
where
    T: 'static,
    A: Node<T>,
{
    node: Box<Option<A>>,
    _t: PhantomData<T>,
}

pub struct PlaceholderSetter<T, A>
where
    T: 'static,
    A: Node<T>,
{
    ptr: *mut Option<A>,
    _t: PhantomData<T>,
}

impl<T, A> Placeholder<T, A>
where
    T: 'static,
    A: Node<T>,
{
    pub fn new(node: Option<A>) -> Self {
        Placeholder {
            node: Box::new(node),
            _t: Default::default(),
        }
    }

    pub fn set(&mut self, node: A) {
        self.node.replace(node);
    }

    pub fn setter(&mut self) -> PlaceholderSetter<T, A> {
        PlaceholderSetter {
            ptr: self.node.as_mut(),
            _t: self._t,
        }
    }
}

impl<T, A> PlaceholderSetter<T, A>
where
    T: 'static,
    A: Node<T>,
{
    pub unsafe fn set(&mut self, node: A) {
        let mut placeholder = Box::from_raw(self.ptr);
        placeholder.replace(node);
        std::mem::forget(placeholder);
    }
}

impl<T, A> Node<T> for Placeholder<T, A>
where
    T: 'static,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.node
            .as_mut()
            .as_mut()
            .expect("Placeholder unset")
            .proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node
            .as_mut()
            .as_mut()
            .expect("Placeholder unset")
            .lock(ctx);
    }

    fn unlock(&mut self) {
        self.node
            .as_mut()
            .as_mut()
            .expect("Placeholder unset")
            .unlock();
    }
}
