use crate::{signal::Signal, Node, ProcContext};

pub struct DownSample<T, A>
where
    T: Signal,
    A: Node<T>,
{
    node: A,
    value: T,
    old_value: T,
    sample_rate: u64,
    next_update: f64,
    pub down_sample_type: DownSampleType,
}

pub enum DownSampleType {
    NearestNeighbor,
    Bilinear,
}

impl<T, A> DownSample<T, A>
where
    T: Signal,
    A: Node<T>,
{
    pub fn new(node: A, value: T, sample_rate: u64, down_sample_type: DownSampleType) -> Self {
        DownSample {
            node,
            old_value: value.clone(),
            value,
            sample_rate,
            next_update: 0.0,
            down_sample_type,
        }
    }
}

impl<T, A> Node<T> for DownSample<T, A>
where
    T: Signal<Float = f64>,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        if self.next_update <= ctx.current_time {
            self.old_value = self.value.clone();
            self.value = self.node.proc(&ProcContext {
                sample_rate: self.sample_rate,
                current_time: ctx.current_time * self.sample_rate as f64 / ctx.sample_rate as f64,
                current_sample: ctx.current_sample * self.sample_rate / ctx.sample_rate,
                rest_proc_samples: ctx.rest_proc_samples * self.sample_rate / ctx.sample_rate,
            });
            self.next_update += 1.0 / self.sample_rate as f64;
        }
        match self.down_sample_type {
            DownSampleType::NearestNeighbor => self.value.clone(),
            DownSampleType::Bilinear => {
                let r = ctx.current_time * self.sample_rate as f64
                    - self.next_update * self.sample_rate as f64
                    + 1.0;
                self.old_value.lerp(&self.value, r)
            }
        }
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}
