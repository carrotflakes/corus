use std::sync::{Arc, Mutex, MutexGuard};

use crate::proc_context::ProcContext;

use super::Node;

pub struct Controllable<T, A>
where
    T: 'static,
    A: Node<T> + 'static,
{
    node: Arc<Mutex<A>>,
    ref_mut: Option<MutexGuard<'static, A>>,
    _t: std::marker::PhantomData<T>,
}

pub struct Controller<T, A>
where
    T: 'static,
    A: Node<T> + 'static,
{
    node: Arc<Mutex<A>>,
    _t: std::marker::PhantomData<T>,
}

impl<T, A> Controllable<T, A>
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

    pub fn controller(&self) -> Controller<T, A> {
        Controller {
            node: self.node.clone(),
            _t: self._t,
        }
    }

    // pub fn borrow_mut(&mut self) -> std::sync::MutexGuard<A> {
    //     if matches!(self.ref_mut, None) {
    //         self.node.lock().unwrap()
    //     } else {
    //         panic!("aaaa");
    //     }
    // }
}

impl<T, A> Node<T> for Controllable<T, A>
where
    T: 'static,
    A: Node<T> + 'static,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.ref_mut
            .as_mut()
            .expect("Controllable unlocked!")
            .proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        if let None = self.ref_mut {
            let mut r = Mutex::lock(&mut self.node).unwrap();
            r.lock(ctx);
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

impl<T, A> Controller<T, A>
where
    T: 'static,
    A: Node<T> + 'static,
{
    pub fn lock(&mut self) -> std::sync::MutexGuard<A> {
        Mutex::lock(&mut self.node).unwrap()
    }
}

impl<T, A> Clone for Controller<T, A>
where
    T: 'static,
    A: Node<T> + 'static,
{
    fn clone(&self) -> Self {
        Self {
            node: self.node.clone(),
            _t: self._t,
        }
    }
}

unsafe impl<T, A> Send for Controllable<T, A>
where
    T: 'static,
    A: Node<T>,
{
}

unsafe impl<T, A> Sync for Controllable<T, A>
where
    T: 'static,
    A: Node<T> + Sync,
{
}
