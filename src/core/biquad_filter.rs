use biquad_filter::types::FilterType as BiquadFilterType;

use crate::signal::{C1f64, C2f64, Mono, Signal, Stereo};

use super::{Node, ProcContext};

pub use biquad_filter::types;

pub struct BiquadFilter<T, N, P>
where
    N: Node<Output = T>,
    N::Output: Signal,
    P: Node<Output = [f64; 6]>,
{
    node: N,
    params: P,
    samples: [N::Output; 4],
}

impl<T, N, P> BiquadFilter<T, N, P>
where
    N: Node<Output = T>,
    N::Output: Signal,
    P: Node<Output = [f64; 6]>,
{
    pub fn new(node: N, params: P) -> Self {
        BiquadFilter {
            node,
            params,
            samples: Default::default(),
        }
    }
}

// TODO: generic
impl<N, P> Node for BiquadFilter<C1f64, N, P>
where
    N: Node<Output = C1f64>,
    P: Node<Output = [f64; 6]>,
{
    type Output = C1f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let [a0, a1, a2, b0, b1, b2] = self.params.proc(ctx);
        let value = self.node.proc(ctx);

        let sample =
            (b0 * value.get_m() + b1 * self.samples[0].get_m() + b2 * self.samples[1].get_m()
                - a1 * self.samples[2].get_m()
                - a2 * self.samples[3].get_m())
                / a0;
        let sample = C1f64::from_m(sample);
        self.samples[1] = self.samples[0];
        self.samples[0] = value;
        self.samples[3] = self.samples[2];
        self.samples[2] = sample;
        sample
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.params.lock(ctx);
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.params.unlock();
        self.node.unlock();
    }
}

impl<N, P> Node for BiquadFilter<C2f64, N, P>
where
    N: Node<Output = C2f64>,
    P: Node<Output = [f64; 6]>,
{
    type Output = C2f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C2f64 {
        let [a0, a1, a2, b0, b1, b2] = self.params.proc(ctx);
        let value = self.node.proc(ctx);

        let sample_l =
            (b0 * value.get_l() + b1 * self.samples[0].get_l() + b2 * self.samples[1].get_l()
                - a1 * self.samples[2].get_l()
                - a2 * self.samples[3].get_l())
                / a0;
        let sample_r =
            (b0 * value.get_r() + b1 * self.samples[0].get_r() + b2 * self.samples[1].get_r()
                - a1 * self.samples[2].get_r()
                - a2 * self.samples[3].get_r())
                / a0;
        let sample = C2f64([sample_l, sample_r]);
        self.samples[1] = self.samples[0];
        self.samples[0] = value;
        self.samples[3] = self.samples[2];
        self.samples[2] = sample;
        sample
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.params.lock(ctx);
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.params.unlock();
        self.node.unlock();
    }
}

pub struct BiquadFilterParams<FT, A, B, C>
where
    FT: BiquadFilterType,
    A: Node<Output = f64>,
    B: Node<Output = f64>,
    C: Node<Output = f64>,
{
    filter_type: FT,
    frequency: A,
    gain: B,
    q: C,
}

impl<FT, A, B, C> BiquadFilterParams<FT, A, B, C>
where
    FT: BiquadFilterType,
    A: Node<Output = f64>,
    B: Node<Output = f64>,
    C: Node<Output = f64>,
{
    pub fn new(filter_type: FT, frequency: A, gain: B, q: C) -> Self {
        Self {
            filter_type,
            frequency,
            gain,
            q,
        }
    }
}

impl<FT, A, B, C> Node for BiquadFilterParams<FT, A, B, C>
where
    FT: BiquadFilterType,
    A: Node<Output = f64>,
    B: Node<Output = f64>,
    C: Node<Output = f64>,
{
    type Output = [f64; 6];

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> [f64; 6] {
        let frequency = self.frequency.proc(ctx);
        let gain = self.gain.proc(ctx);
        let q = self.q.proc(ctx);
        self.filter_type.compute_params_from_frequency(
            ctx.sample_rate,
            frequency.get_m(),
            gain.get_m(),
            q.get_m(),
        )
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.frequency.lock(ctx);
        self.gain.lock(ctx);
        self.q.lock(ctx);
    }

    fn unlock(&mut self) {
        self.frequency.unlock();
        self.gain.unlock();
        self.q.unlock();
    }
}
