use std::marker::PhantomData;

use crate::signal::{C1f64, C2f64, Signal, Mono, Stereo};

use super::{Node, ProcContext};

pub struct BiquadFilter<FT, T, N, A, B, C, DN, DA, DB, DC>
where
    FT: BiquadFilterType,
    T: Clone + 'static + std::ops::Add<Output = T> + Signal + Default,
    N: Node<T> + ?Sized,
    A: Node<C1f64> + ?Sized,
    B: Node<C1f64> + ?Sized,
    C: Node<C1f64> + ?Sized,
    DN: AsMut<N>,
    DA: AsMut<A>,
    DB: AsMut<B>,
    DC: AsMut<C>,
{
    filter_type: FT,
    node: DN,
    frequency: DA,
    gain: DB,
    q: DC,
    samples: [T; 4],
    _t: (
        PhantomData<T>,
        PhantomData<N>,
        PhantomData<A>,
        PhantomData<B>,
        PhantomData<C>,
    ),
}

impl<FT, T, N, A, B, C, DN, DA, DB, DC> BiquadFilter<FT, T, N, A, B, C, DN, DA, DB, DC>
where
    FT: BiquadFilterType,
    T: Clone + 'static + std::ops::Add<Output = T> + Signal + Default,
    N: Node<T> + ?Sized,
    A: Node<C1f64> + ?Sized,
    B: Node<C1f64> + ?Sized,
    C: Node<C1f64> + ?Sized,
    DN: AsMut<N>,
    DA: AsMut<A>,
    DB: AsMut<B>,
    DC: AsMut<C>,
{
    pub fn new(filter_type: FT, node: DN, frequency: DA, gain: DB, q: DC) -> Self {
        BiquadFilter {
            filter_type,
            node,
            frequency,
            gain,
            q,
            samples: Default::default(),
            _t: Default::default(),
        }
    }
}

// TODO: generic
impl<FT, N, A, B, C, DN, DA, DB, DC> Node<C1f64>
    for BiquadFilter<FT, C1f64, N, A, B, C, DN, DA, DB, DC>
where
    FT: BiquadFilterType,
    N: Node<C1f64> + ?Sized,
    A: Node<C1f64> + ?Sized,
    B: Node<C1f64> + ?Sized,
    C: Node<C1f64> + ?Sized,
    DN: AsMut<N>,
    DA: AsMut<A>,
    DB: AsMut<B>,
    DC: AsMut<C>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let frequency = self.frequency.as_mut().proc(ctx);
        let gain = self.gain.as_mut().proc(ctx);
        let q = self.q.as_mut().proc(ctx);
        let value = self.node.as_mut().proc(ctx);
        let [a0, a1, a2, b0, b1, b2] =
            self.filter_type
                .compute_params(ctx.sample_rate, frequency.get_m(), gain.get_m(), q.get_m());

        let sample = ((b0 / a0) * value.get_m()
            + (b1 / a0) * self.samples[0].get_m()
            + (b2 / a0) * self.samples[1].get_m())
            - (a1 / a0) * self.samples[2].get_m()
            - (a2 / a0) * self.samples[3].get_m();
        let sample = C1f64::from_m(sample);
        self.samples[1] = self.samples[0];
        self.samples[0] = value;
        self.samples[3] = self.samples[2];
        self.samples[2] = sample;
        sample
    }

    fn lock(&mut self) {
        self.node.as_mut().lock();
        self.frequency.as_mut().lock();
        self.gain.as_mut().lock();
        self.q.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.node.as_mut().unlock();
        self.frequency.as_mut().unlock();
        self.gain.as_mut().unlock();
        self.q.as_mut().unlock();
    }
}

impl<FT, N, A, B, C, DN, DA, DB, DC> Node<C2f64>
    for BiquadFilter<FT, C2f64, N, A, B, C, DN, DA, DB, DC>
where
    FT: BiquadFilterType,
    N: Node<C2f64> + ?Sized,
    A: Node<C1f64> + ?Sized,
    B: Node<C1f64> + ?Sized,
    C: Node<C1f64> + ?Sized,
    DN: AsMut<N>,
    DA: AsMut<A>,
    DB: AsMut<B>,
    DC: AsMut<C>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C2f64 {
        let frequency = self.frequency.as_mut().proc(ctx);
        let gain = self.gain.as_mut().proc(ctx);
        let q = self.q.as_mut().proc(ctx);
        let value = self.node.as_mut().proc(ctx);
        let [a0, a1, a2, b0, b1, b2] =
            self.filter_type
                .compute_params(ctx.sample_rate, frequency.get_m(), gain.get_m(), q.get_m());

        let sample_l = ((b0 / a0) * value.get_l()
            + (b1 / a0) * self.samples[0].get_l()
            + (b2 / a0) * self.samples[1].get_l())
            - (a1 / a0) * self.samples[2].get_l()
            - (a2 / a0) * self.samples[3].get_l();
        let sample_r = ((b0 / a0) * value.get_r()
            + (b1 / a0) * self.samples[0].get_r()
            + (b2 / a0) * self.samples[1].get_r())
            - (a1 / a0) * self.samples[2].get_r()
            - (a2 / a0) * self.samples[3].get_r();
        let sample = C2f64([sample_l, sample_r]);
        self.samples[1] = self.samples[0];
        self.samples[0] = value;
        self.samples[3] = self.samples[2];
        self.samples[2] = sample;
        sample
    }

    fn lock(&mut self) {
        self.node.as_mut().lock();
        self.frequency.as_mut().lock();
        self.gain.as_mut().lock();
        self.q.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.node.as_mut().unlock();
        self.frequency.as_mut().unlock();
        self.gain.as_mut().unlock();
        self.q.as_mut().unlock();
    }
}

impl<FT, T, N, A, B, C, DN, DA, DB, DC> AsMut<Self>
    for BiquadFilter<FT, T, N, A, B, C, DN, DA, DB, DC>
where
    FT: BiquadFilterType,
    T: Clone + 'static + std::ops::Add<Output = T> + Signal + Default,
    N: Node<T> + ?Sized,
    A: Node<C1f64> + ?Sized,
    B: Node<C1f64> + ?Sized,
    C: Node<C1f64> + ?Sized,
    DN: AsMut<N>,
    DA: AsMut<A>,
    DB: AsMut<B>,
    DC: AsMut<C>,
{
    fn as_mut(&mut self) -> &mut BiquadFilter<FT, T, N, A, B, C, DN, DA, DB, DC> {
        self
    }
}

pub trait BiquadFilterType {
    fn compute_params(&self, sample_rate: u64, frequency: f64, gain: f64, q: f64) -> [f64; 6];
}

#[derive(Debug, Clone)]
pub struct LowPass;

impl BiquadFilterType for LowPass {
    #[inline]
    fn compute_params(&self, sample_rate: u64, frequency: f64, _gain: f64, q: f64) -> [f64; 6] {
        // let a = 10.0f64.powf(gain / 40.0);
        let w0 = (2.0 * std::f64::consts::PI * frequency) / sample_rate as f64;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        [
            1.0 + alpha,
            -2.0 * cos,
            1.0 - alpha,
            (1.0 - cos) / 2.0,
            1.0 - cos,
            (1.0 - cos) / 2.0,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct HighPass;

impl BiquadFilterType for HighPass {
    #[inline]
    fn compute_params(&self, sample_rate: u64, frequency: f64, _gain: f64, q: f64) -> [f64; 6] {
        // let a = 10.0f64.powf(gain / 40.0);
        let w0 = (2.0 * std::f64::consts::PI * frequency) / sample_rate as f64;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        [
            1.0 + alpha,
            -2.0 * cos,
            1.0 - alpha,
            (1.0 + cos) / 2.0,
            -(1.0 + cos),
            (1.0 + cos) / 2.0,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct BandPass;

impl BiquadFilterType for BandPass {
    #[inline]
    fn compute_params(&self, sample_rate: u64, frequency: f64, _gain: f64, q: f64) -> [f64; 6] {
        // let a = 10.0f64.powf(gain / 40.0);
        let w0 = (2.0 * std::f64::consts::PI * frequency) / sample_rate as f64;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        [
            1.0 + alpha,
            -2.0 * cos,
            1.0 - alpha,
            sin / 2.0,
            0.0,
            -sin / 2.0,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct LowShelf;

impl BiquadFilterType for LowShelf {
    #[inline]
    fn compute_params(&self, sample_rate: u64, frequency: f64, gain: f64, q: f64) -> [f64; 6] {
        let a = 10.0f64.powf(gain / 40.0);
        let w0 = (2.0 * std::f64::consts::PI * frequency) / sample_rate as f64;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        let x = a.sqrt() * 2.0 * alpha;
        [
            (a + 1.0) + (a - 1.0) * cos + x,
            -2.0 * ((a - 1.0) + (a + 1.0) * cos),
            (a + 1.0) + (a - 1.0) * cos - x,
            a * ((a + 1.0) - (a - 1.0) * cos + x),
            2.0 * a * ((a - 1.0) - (a + 1.0) * cos),
            a * ((a + 1.0) - (a - 1.0) * cos - x),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct HighShelf;

impl BiquadFilterType for HighShelf {
    #[inline]
    fn compute_params(&self, sample_rate: u64, frequency: f64, gain: f64, q: f64) -> [f64; 6] {
        let a = 10.0f64.powf(gain / 40.0);
        let w0 = (2.0 * std::f64::consts::PI * frequency) / sample_rate as f64;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        let x = a.sqrt() * 2.0 * alpha;
        [
            (a + 1.0) - (a - 1.0) * cos + x,
            2.0 * ((a - 1.0) - (a + 1.0) * cos),
            (a + 1.0) - (a - 1.0) * cos - x,
            a * ((a + 1.0) + (a - 1.0) * cos + x),
            -2.0 * a * ((a - 1.0) + (a + 1.0) * cos),
            a * ((a + 1.0) + (a - 1.0) * cos - x),
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Peaking;

impl BiquadFilterType for Peaking {
    #[inline]
    fn compute_params(&self, sample_rate: u64, frequency: f64, gain: f64, q: f64) -> [f64; 6] {
        let a = 10.0f64.powf(gain / 40.0);
        let w0 = (2.0 * std::f64::consts::PI * frequency) / sample_rate as f64;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        [
            1.0 + alpha / a,
            -2.0 * cos,
            1.0 - alpha / a,
            1.0 + alpha * a,
            -2.0 * cos,
            1.0 - alpha * a,
        ]
    }
}

#[derive(Debug, Clone)]
pub struct Notch;

impl BiquadFilterType for Notch {
    #[inline]
    fn compute_params(&self, sample_rate: u64, frequency: f64, _gain: f64, q: f64) -> [f64; 6] {
        // let a = 10.0f64.powf(gain / 40.0);
        let w0 = (2.0 * std::f64::consts::PI * frequency) / sample_rate as f64;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        [1.0 + alpha, -2.0 * cos, 1.0 - alpha, 1.0, -2.0 * cos, 1.0]
    }
}

#[derive(Debug, Clone)]
pub struct AllPass;

impl BiquadFilterType for AllPass {
    #[inline]
    fn compute_params(&self, sample_rate: u64, frequency: f64, _gain: f64, q: f64) -> [f64; 6] {
        // let a = 10.0f64.powf(gain / 40.0);
        let w0 = (2.0 * std::f64::consts::PI * frequency) / sample_rate as f64;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        [
            1.0 + alpha,
            -2.0 * cos,
            1.0 - alpha,
            1.0 - alpha,
            -2.0 * cos,
            1.0 + alpha,
        ]
    }
}

#[derive(Debug, Clone)]
pub enum BiquadFilterTypeDynamic {
    LowPass,
    HighPass,
    BandPass,
    LowShelf,
    HighShelf,
    Peaking,
    Notch,
    AllPass,
}

impl BiquadFilterType for BiquadFilterTypeDynamic {
    #[inline]
    fn compute_params(&self, sample_rate: u64, frequency: f64, gain: f64, q: f64) -> [f64; 6] {
        match self {
            BiquadFilterTypeDynamic::LowPass => {
                LowShelf::compute_params(&LowShelf, sample_rate, frequency, gain, q)
            }
            BiquadFilterTypeDynamic::HighPass => {
                HighPass::compute_params(&HighPass, sample_rate, frequency, gain, q)
            }
            BiquadFilterTypeDynamic::BandPass => {
                BandPass::compute_params(&BandPass, sample_rate, frequency, gain, q)
            }
            BiquadFilterTypeDynamic::LowShelf => {
                LowShelf::compute_params(&LowShelf, sample_rate, frequency, gain, q)
            }
            BiquadFilterTypeDynamic::HighShelf => {
                HighShelf::compute_params(&HighShelf, sample_rate, frequency, gain, q)
            }
            BiquadFilterTypeDynamic::Peaking => {
                Peaking::compute_params(&Peaking, sample_rate, frequency, gain, q)
            }
            BiquadFilterTypeDynamic::Notch => {
                Notch::compute_params(&Notch, sample_rate, frequency, gain, q)
            }
            BiquadFilterTypeDynamic::AllPass => {
                AllPass::compute_params(&AllPass, sample_rate, frequency, gain, q)
            }
        }
    }
}
