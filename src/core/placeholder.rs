use crate::proc_context::ProcContext;

use super::Node;

pub struct Placeholder<A>
where
    A: Node,
{
    node: Box<Option<A>>,
}

pub struct PlaceholderSetter<A>
where
    A: Node,
{
    ptr: *mut Option<A>,
}

impl<A> Placeholder<A>
where
    A: Node,
{
    pub fn new(node: Option<A>) -> Self {
        Placeholder {
            node: Box::new(node),
        }
    }

    pub fn set(&mut self, node: A) {
        self.node.replace(node);
    }

    pub fn setter(&mut self) -> PlaceholderSetter<A> {
        PlaceholderSetter {
            ptr: self.node.as_mut(),
        }
    }
}

impl<A> PlaceholderSetter<A>
where
    A: Node,
{
    pub unsafe fn set(&mut self, node: A) {
        let mut placeholder = Box::from_raw(self.ptr);
        placeholder.replace(node);
        std::mem::forget(placeholder);
    }
}

impl<A> Node for Placeholder<A>
where
    A: Node,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
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
