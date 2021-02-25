use std::marker::PhantomData;

use crate::Node;

pub struct ProcContext {
    pub sample_rate: u64,
    pub time: f64,
}

impl ProcContext {
    pub fn new(sample_rate: u64) -> Self {
        ProcContext {
            sample_rate,
            time: 0.0,
        }
    }

    #[inline]
    pub fn sample<T: 'static, N: Node<T> + ?Sized>(&mut self, mut node: impl AsMut<N>) -> T {
        let r = node.as_mut().proc(self);
        self.time += 1.0 / self.sample_rate as f64;
        r
    }

    #[inline]
    pub fn lock<'a, T: 'static, A: Node<T> + ?Sized, DA: AsMut<A>>(
        &'a mut self,
        node: DA,
    ) -> ProcGuard<'a, T, A, DA> {
        ProcGuard::new(self, node)
    }
}

pub struct ProcGuard<'a, T: 'static, A: Node<T> + ?Sized, DA: AsMut<A>> {
    context: &'a mut ProcContext,
    node: DA,
    _t: (PhantomData<T>, PhantomData<A>),
}

impl<'a, T: 'static, A: Node<T> + ?Sized, DA: AsMut<A>> ProcGuard<'a, T, A, DA> {
    fn new(context: &'a mut ProcContext, mut node: DA) -> Self {
        node.as_mut().lock();
        Self {
            context,
            node,
            _t: Default::default(),
        }
    }

    #[inline]
    pub fn sample(&mut self) -> T {
        let r = self.node.as_mut().proc(self.context);
        self.context.time += 1.0 / self.context.sample_rate as f64;
        r
    }
}

impl<'a, T: 'static, A: Node<T> + ?Sized, DA: AsMut<A>> Drop for ProcGuard<'a, T, A, DA> {
    fn drop(&mut self) {
        self.node.as_mut().unlock();
    }
}

impl<'a, T: 'static, A: Node<T> + ?Sized, DA: AsMut<A>> Iterator for ProcGuard<'a, T, A, DA> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.sample())
    }
}
