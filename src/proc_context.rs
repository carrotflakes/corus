use std::marker::PhantomData;

use crate::Node;

pub struct ProcContext {
    pub sample_rate: u64, // DO NOT change after construct!
    pub current_time: f64,
    pub current_sample: u64,
    pub proc_samples: u64, // TODO
    pub proc_length: f64, // TODO
}

impl ProcContext {
    pub fn new(sample_rate: u64) -> Self {
        ProcContext {
            sample_rate,
            current_time: 0.0,
            current_sample: 0,
            proc_samples: 1,
            proc_length: 1.0 / sample_rate as f64,
        }
    }

    #[inline]
    pub fn sample<T: 'static, N: Node<T> + ?Sized>(&mut self, node: &mut N) -> T {
        let r = node.proc(self);
        self.current_sample += 1;
        self.current_time = self.current_sample as f64 / self.sample_rate as f64;
        r
    }

    #[inline]
    pub fn lock<'a, T: 'static, A: Node<T> + ?Sized>(
        &'a mut self,
        node: &'a mut A,
    ) -> ProcGuard<'a, T, A> {
        ProcGuard::new(self, node)
    }
}

pub struct ProcGuard<'a, T: 'static, A: Node<T> + ?Sized> {
    context: &'a mut ProcContext,
    node: &'a mut A,
    _t: PhantomData<T>,
}

impl<'a, T: 'static, A: Node<T> + ?Sized> ProcGuard<'a, T, A> {
    fn new(context: &'a mut ProcContext, node: &'a mut A) -> Self {
        node.lock();
        Self {
            context,
            node,
            _t: Default::default(),
        }
    }

    #[inline]
    pub fn sample(&mut self) -> T {
        let r = self.node.proc(self.context);
        self.context.current_time += 1.0 / self.context.sample_rate as f64;
        r
    }
}

impl<'a, T: 'static, A: Node<T> + ?Sized> Drop for ProcGuard<'a, T, A> {
    fn drop(&mut self) {
        self.node.unlock();
    }
}

impl<'a, T: 'static, A: Node<T> + ?Sized> Iterator for ProcGuard<'a, T, A> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.sample())
    }
}
