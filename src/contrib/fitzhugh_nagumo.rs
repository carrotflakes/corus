use crate::{signal::C1f64, Node};

pub struct FitzhughNagumo<A, B>
where
    A: Node<Output = C1f64>,
    B: Node<Output = C1f64>,
{
    pitch: A, // TODO: naming
    input: B,
    v: C1f64,
    w: C1f64,
}

impl<A, B> FitzhughNagumo<A, B>
where
    A: Node<Output = C1f64>,
    B: Node<Output = C1f64>,
{
    pub fn new(pitch: A, input: B) -> Self {
        Self {
            pitch,
            input,
            v: 0.0,
            w: 0.0,
        }
    }
}

impl<A, B> Node for FitzhughNagumo<A, B>
where
    A: Node<Output = C1f64>,
    B: Node<Output = C1f64>,
{
    type Output = C1f64;

    fn proc(&mut self, ctx: &crate::ProcContext) -> Self::Output {
        let pitch = self.pitch.proc(ctx) / ctx.sample_rate as f64;
        let input = self.input.proc(ctx);

        let v = self.v;
        let w = self.w;

        self.v += pitch * (v - v.powi(3) / 3.0 - w + input);
        self.w += pitch * 0.08 * (v + 0.7 - 0.8 * w);

        self.v
    }

    fn lock(&mut self, ctx: &crate::ProcContext) {
        self.pitch.lock(ctx);
        self.input.lock(ctx);
    }

    fn unlock(&mut self) {
        self.pitch.unlock();
        self.input.unlock();
    }
}
