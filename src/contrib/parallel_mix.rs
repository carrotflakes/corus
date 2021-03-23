use std::{ops::Add, thread};

use crate::{EventQueue, Node, ProcContext};

// Non blocking
pub struct ParallelMix<A>
where
    A: Node + Send + Sync + 'static,
    A::Output: Clone + Add<Output = A::Output> + Default + Send + Sync,
{
    nodes: Vec<A>,
    event_queues: Vec<EventQueue>,
    samples: Vec<A::Output>,
    progresses: Vec<usize>,
    i: usize,
    len: usize,
}

impl<A> ParallelMix<A>
where
    A: Node + Send + Sync + 'static,
    A::Output: Clone + Add<Output = A::Output> + Default + Send + Sync,
{
    pub fn new(nodes: Vec<A>) -> Self {
        ParallelMix {
            event_queues: (0..nodes.len()).map(|_| EventQueue::new()).collect(),
            nodes,
            samples: Vec::new(),
            progresses: Vec::new(),
            i: 0,
            len: 0,
        }
    }
}

impl<A> Node for ParallelMix<A>
where
    A: Node + Send + Sync + 'static,
    A::Output: Clone + Add<Output = A::Output> + Default + Send + Sync,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, _ctx: &ProcContext) -> Self::Output {
        let mut v = Default::default();
        for i in 0..self.nodes.len() {
            while self.progresses[i] <= self.i {
                thread::sleep(std::time::Duration::from_millis(1));
            }
            v = v + self.samples[i * self.len + self.i].clone();
        }
        self.i += 1;
        v
    }

    fn lock(&mut self, ctx: &ProcContext) {
        for node in &mut self.nodes {
            node.lock(ctx);
        }

        self.samples.resize_with(
            self.nodes.len() * ctx.rest_proc_samples as usize,
            Default::default,
        );
        self.progresses.resize(self.nodes.len(), 0);
        self.progresses.fill(0);
        self.i = 0;
        self.len = ctx.rest_proc_samples as usize;
        for (i, node) in self.nodes.iter_mut().enumerate() {
            let node = unsafe { std::mem::transmute::<_, &'static mut A>(node) };
            let mut ctx = ctx.clone();
            ctx.event_queue = self.event_queues[i].clone();
            let mut samples = unsafe {
                std::mem::transmute::<_, &'static mut [Self::Output]>(
                    &mut self.samples[i * ctx.rest_proc_samples as usize
                        ..(i + 1) * ctx.rest_proc_samples as usize],
                )
            };
            let progress =
                unsafe { std::mem::transmute::<_, &'static mut usize>(&mut self.progresses[i]) };
            thread::spawn(move || {
                while ctx.rest_proc_samples > 0 {
                    samples[0] = node.proc(&ctx);
                    samples = &mut samples[1..];
                    ctx.current_sample += 1;
                    ctx.current_time = ctx.current_sample as f64 / ctx.sample_rate as f64;
                    ctx.rest_proc_samples -= 1;
                    *progress += 1;
                }
            });
        }
    }

    fn unlock(&mut self) {
        for node in &mut self.nodes {
            node.unlock();
        }
    }
}
