use super::{Node, ProcContext};

pub struct Sine<A>
where
    A: Node<Output = f64>,
{
    frequency: A,
    phase: f64,
}

impl<A> Sine<A>
where
    A: Node<Output = f64>,
{
    pub fn new(frequency: A) -> Self {
        Sine {
            frequency,
            phase: 0.0.into(),
        }
    }
}

impl<A> Node for Sine<A>
where
    A: Node<Output = f64>,
{
    type Output = A::Output;

    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let f = self.frequency.proc(ctx);
        let p = self.phase;
        self.phase = (self.phase + f / ctx.sample_rate as f64)
            .fract()
            .into();
        (p * std::f64::consts::PI * 2.0).sin().into()
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.frequency.lock(ctx);
    }

    fn unlock(&mut self) {
        self.frequency.unlock();
    }
}
