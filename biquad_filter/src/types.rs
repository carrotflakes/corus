use std::f64::consts::TAU;

pub trait BiquadFilterType {
    fn compute_params(&self, period: f64, gain: f64, q: f64) -> [f64; 6];

    #[inline]
    fn compute_params_from_frequency(
        &self,
        sample_rate: u64,
        frequency: f64,
        gain: f64,
        q: f64,
    ) -> [f64; 6] {
        self.compute_params(frequency / sample_rate as f64, gain, q)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LowPass;

impl BiquadFilterType for LowPass {
    #[inline]
    fn compute_params(&self, period: f64, _gain: f64, q: f64) -> [f64; 6] {
        let w0 = TAU * period;
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

#[derive(Debug, Clone, Copy)]
pub struct HighPass;

impl BiquadFilterType for HighPass {
    #[inline]
    fn compute_params(&self, period: f64, _gain: f64, q: f64) -> [f64; 6] {
        let w0 = TAU * period;
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

#[derive(Debug, Clone, Copy)]
pub struct BandPass;

impl BiquadFilterType for BandPass {
    #[inline]
    fn compute_params(&self, period: f64, _gain: f64, q: f64) -> [f64; 6] {
        let w0 = TAU * period;
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

#[derive(Debug, Clone, Copy)]
pub struct LowShelf;

impl BiquadFilterType for LowShelf {
    #[inline]
    fn compute_params(&self, period: f64, gain: f64, q: f64) -> [f64; 6] {
        let w0 = TAU * period;
        let a = gain;
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

#[derive(Debug, Clone, Copy)]
pub struct HighShelf;

impl BiquadFilterType for HighShelf {
    #[inline]
    fn compute_params(&self, period: f64, gain: f64, q: f64) -> [f64; 6] {
        let w0 = TAU * period;
        let a = gain;
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

#[derive(Debug, Clone, Copy)]
pub struct Peaking;

impl BiquadFilterType for Peaking {
    #[inline]
    fn compute_params(&self, period: f64, gain: f64, q: f64) -> [f64; 6] {
        let w0 = TAU * period;
        let a = gain;
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

#[derive(Debug, Clone, Copy)]
pub struct Notch;

impl BiquadFilterType for Notch {
    #[inline]
    fn compute_params(&self, period: f64, _gain: f64, q: f64) -> [f64; 6] {
        let w0 = TAU * period;
        let (sin, cos) = w0.sin_cos();
        let alpha = sin / (2.0 * q);
        [1.0 + alpha, -2.0 * cos, 1.0 - alpha, 1.0, -2.0 * cos, 1.0]
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AllPass;

impl BiquadFilterType for AllPass {
    #[inline]
    fn compute_params(&self, period: f64, _gain: f64, q: f64) -> [f64; 6] {
        let w0 = TAU * period;
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

#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
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
    fn compute_params(&self, period: f64, gain: f64, q: f64) -> [f64; 6] {
        match self {
            BiquadFilterTypeDynamic::LowPass => LowShelf.compute_params(period, gain, q),
            BiquadFilterTypeDynamic::HighPass => HighPass.compute_params(period, gain, q),
            BiquadFilterTypeDynamic::BandPass => BandPass.compute_params(period, gain, q),
            BiquadFilterTypeDynamic::LowShelf => LowShelf.compute_params(period, gain, q),
            BiquadFilterTypeDynamic::HighShelf => HighShelf.compute_params(period, gain, q),
            BiquadFilterTypeDynamic::Peaking => Peaking.compute_params(period, gain, q),
            BiquadFilterTypeDynamic::Notch => Notch.compute_params(period, gain, q),
            BiquadFilterTypeDynamic::AllPass => AllPass.compute_params(period, gain, q),
        }
    }
}
