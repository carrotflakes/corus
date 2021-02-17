use std::{cell::RefCell, rc::Rc, sync::{Arc, Mutex}};

use super::{Node, ProcContext};

pub struct Share<T, A>
where
    T: 'static,
    A: Node<T>,
{
    node: Rc<RefCell<A>>,
    _t: std::marker::PhantomData<T>,
}

impl<T, A> Share<T, A>
where
    T: 'static,
    A: Node<T>,
{
    pub fn new(node: A) -> Self {
        Self {
            node: Rc::new(RefCell::new(node)),
            _t: Default::default(),
        }
    }

    pub fn borrow_mut(&mut self) -> std::cell::RefMut<A> {
        self.node.borrow_mut()
    }
}

impl<T, A> Node<T> for Share<T, A>
where
    T: 'static,
    A: Node<T>,
{
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let mut node = self.node.borrow_mut();
        node.proc(ctx)
    }
}

impl<T, A> AsMut<Self> for Share<T, A>
where
    T: 'static,
    A: Node<T>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, A> Clone for Share<T, A>
where
    T: 'static,
    A: Node<T>,
{
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            _t: self._t.clone(),
        }
    }
}



pub struct ShareC<T, A>
where
    T: 'static,
    A: Node<T>,
{
    node: Arc<Mutex<A>>,
    _t: std::marker::PhantomData<T>,
}

impl<T, A> ShareC<T, A>
where
    T: 'static,
    A: Node<T>,
{
    pub fn new(node: A) -> Self {
        Self {
            node: Arc::new(Mutex::new(node)),
            _t: Default::default(),
        }
    }

    pub fn borrow_mut(&mut self) -> std::sync::MutexGuard<A> {
        self.node.lock().unwrap()
    }
}

impl<T, A> Node<T> for ShareC<T, A>
where
    T: 'static,
    A: Node<T>,
{
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let mut node = self.node.lock().unwrap();
        node.proc(ctx)
    }
}

impl<T, A> AsMut<Self> for ShareC<T, A>
where
    T: 'static,
    A: Node<T>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<T, A> Clone for ShareC<T, A>
where
    T: 'static,
    A: Node<T>,
{
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            _t: self._t.clone(),
        }
    }
}
