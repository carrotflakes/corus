use std::{
    cell::{RefCell, RefMut},
    rc::Rc,
    sync::{Arc, Mutex, MutexGuard},
};

use super::{Node, ProcContext};

pub struct Share<T, A>
where
    T: 'static,
    A: Node<T> + 'static,
{
    node: Rc<RefCell<A>>,
    ref_mut: Option<std::cell::RefMut<'static, A>>,
    _t: std::marker::PhantomData<T>,
}

impl<T, A> Share<T, A>
where
    T: 'static,
    A: Node<T> + 'static,
{
    pub fn new(node: A) -> Self {
        Self {
            node: Rc::new(RefCell::new(node)),
            ref_mut: None,
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
    A: Node<T> + 'static,
{
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.ref_mut.as_mut().expect("Share unlocked!").proc(ctx)
    }

    fn lock(&mut self) {
        if let None = self.ref_mut {
            let mut r = self.node.borrow_mut();
            r.lock();
            self.ref_mut = Some(unsafe { std::mem::transmute::<_, RefMut<'static, A>>(r) });
        }
    }

    fn unlock(&mut self) {
        if let Some(r) = &mut self.ref_mut {
            r.unlock();
        }
        self.ref_mut = None;
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
            ref_mut: None,
            _t: self._t.clone(),
        }
    }
}

pub struct ShareC<T, A>
where
    T: 'static,
    A: Node<T> + 'static,
{
    node: Arc<Mutex<A>>,
    ref_mut: Option<MutexGuard<'static, A>>,
    _t: std::marker::PhantomData<T>,
}

impl<T, A> ShareC<T, A>
where
    T: 'static,
    A: Node<T> + 'static,
{
    pub fn new(node: A) -> Self {
        Self {
            node: Arc::new(Mutex::new(node)),
            ref_mut: None,
            _t: Default::default(),
        }
    }

    pub fn borrow_mut(&mut self) -> std::sync::MutexGuard<A> {
        if matches!(self.ref_mut, None) {
            panic!("ShareC unlocked");
        } else {
            self.node.lock().unwrap()
        }
    }
}

impl<T, A> Node<T> for ShareC<T, A>
where
    T: 'static,
    A: Node<T> + 'static,
{
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.ref_mut.as_mut().expect("ShareC unlocked!").proc(ctx)
    }

    fn lock(&mut self) {
        if let None = self.ref_mut {
            let mut r = self.node.lock().unwrap();
            r.lock();
            self.ref_mut = Some(unsafe { std::mem::transmute::<_, MutexGuard<'static, A>>(r) });
        }
    }

    fn unlock(&mut self) {
        if let Some(r) = &mut self.ref_mut {
            r.unlock();
        }
        self.ref_mut = None;
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
            ref_mut: None,
            _t: self._t.clone(),
        }
    }
}

unsafe impl<T, A> Send for ShareC<T, A>
where
    T: 'static,
    A: Node<T>,
    {}
