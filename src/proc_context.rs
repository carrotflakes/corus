use crate::{EventQueue, Node, time::AsSample};

#[derive(Clone)]
pub struct ProcContext {
    pub sample_rate: u64, // DO NOT change after construct!
    pub current_time: f64,
    pub current_sample: u64,
    pub rest_proc_samples: u64,
    pub event_queue: EventQueue,
}

impl ProcContext {
    pub fn new(sample_rate: u64) -> Self {
        ProcContext {
            sample_rate,
            current_time: 0.0,
            current_sample: 0,
            rest_proc_samples: 0,
            event_queue: EventQueue::new(),
        }
    }

    #[inline]
    pub fn lock<'a, A: Node + ?Sized, S: AsSample>(
        &'a mut self,
        node: &'a mut A,
        proc_length: S,
    ) -> ProcGuard<'a, A> {
        self.rest_proc_samples = proc_length.as_sample(self.sample_rate);
        ProcGuard::new(self, node)
    }
}

pub struct ProcGuard<'a, A: Node + ?Sized> {
    context: &'a mut ProcContext,
    node: &'a mut A,
}

impl<'a, A: Node + ?Sized> ProcGuard<'a, A> {
    fn new(context: &'a mut ProcContext, node: &'a mut A) -> Self {
        node.lock(context);
        Self {
            context,
            node,
        }
    }

    #[inline]
    pub fn sample(&mut self) -> A::Output {
        if self.context.rest_proc_samples == 0 {
            panic!("Exceeded the allowed number of samples");
        }
        self.context.event_queue.dispatch(self.context.current_time);
        let r = self.node.proc(self.context);
        self.context.current_sample += 1;
        self.context.current_time = self.context.current_sample as f64 / self.context.sample_rate as f64;
        self.context.rest_proc_samples -= 1;
        r
    }
}

impl<'a, A: Node + ?Sized> Drop for ProcGuard<'a, A> {
    fn drop(&mut self) {
        self.node.unlock();
    }
}

impl<'a, A: Node + ?Sized> Iterator for ProcGuard<'a, A> {
    type Item = A::Output;

    fn next(&mut self) -> Option<Self::Item> {
        if self.context.rest_proc_samples != 0 {
            Some(self.sample())
        } else {
            None
        }
    }
}
