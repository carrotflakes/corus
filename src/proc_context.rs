use std::marker::PhantomData;

use crate::{time::AsSample, Node};

#[derive(Clone)]
pub struct ProcContext {
    pub sample_rate: u64, // DO NOT change after construct!
    pub current_time: f64,
    pub current_sample: u64,
    pub rest_proc_samples: u64,
}

impl ProcContext {
    pub fn new(sample_rate: u64) -> Self {
        ProcContext {
            sample_rate,
            current_time: 0.0,
            current_sample: 0,
            rest_proc_samples: 0,
        }
    }

    #[inline]
    pub fn lock<'a, T: 'static, A: Node<T> + ?Sized, S: AsSample>(
        &'a mut self,
        node: &'a mut A,
        proc_length: S,
    ) -> ProcGuard<'a, T, A> {
        self.rest_proc_samples = proc_length.as_sample(self.sample_rate);
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
        node.lock(context);
        Self {
            context,
            node,
            _t: Default::default(),
        }
    }

    #[inline]
    pub fn sample(&mut self) -> T {
        if self.context.rest_proc_samples == 0 {
            panic!("Exceeded the allowed number of samples");
        }
        let r = self.node.proc(self.context);
        self.context.current_sample += 1;
        self.context.current_time = self.context.current_sample as f64 / self.context.sample_rate as f64;
        self.context.rest_proc_samples -= 1;
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
        if self.context.rest_proc_samples != 0 {
            Some(self.sample())
        } else {
            None
        }
    }
}
