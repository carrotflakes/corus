use crate::signal::{C1f64, Mono};

use super::{Node, ProcContext};

pub struct Sine<A>
where
    A: Node<C1f64>,
{
    frequency: A,
    phase: C1f64,
}

impl<A> Sine<A>
where
    A: Node<C1f64>,
{
    pub fn new(frequency: A) -> Self {
        Sine {
            frequency,
            phase: 0.0.into(),
        }
    }
}

impl<A> Node<C1f64> for Sine<A>
where
    A: Node<C1f64>,
{
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let f = self.frequency.proc(ctx);
        let p = self.phase;
        self.phase = (self.phase.get_m() + f.get_m() / ctx.sample_rate as f64)
            .fract()
            .into();
        (p.get_m() * std::f64::consts::PI * 2.0).sin().into()
    }

    fn lock(&mut self) {
        self.frequency.lock();
    }

    fn unlock(&mut self) {
        self.frequency.unlock();
    }
}
