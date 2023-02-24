use std::f64::consts::TAU;

pub trait FilterType {
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

impl FilterType for LowPass {
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

impl FilterType for HighPass {
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

impl FilterType for BandPass {
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

impl FilterType for LowShelf {
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

impl FilterType for HighShelf {
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

impl FilterType for Peaking {
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

impl FilterType for Notch {
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

impl FilterType for AllPass {
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
pub enum FilterTypeDynamic {
    LowPass,
    HighPass,
    BandPass,
    LowShelf,
    HighShelf,
    Peaking,
    Notch,
    AllPass,
}

impl FilterType for FilterTypeDynamic {
    #[inline]
    fn compute_params(&self, period: f64, gain: f64, q: f64) -> [f64; 6] {
        match self {
            FilterTypeDynamic::LowPass => LowShelf.compute_params(period, gain, q),
            FilterTypeDynamic::HighPass => HighPass.compute_params(period, gain, q),
            FilterTypeDynamic::BandPass => BandPass.compute_params(period, gain, q),
            FilterTypeDynamic::LowShelf => LowShelf.compute_params(period, gain, q),
            FilterTypeDynamic::HighShelf => HighShelf.compute_params(period, gain, q),
            FilterTypeDynamic::Peaking => Peaking.compute_params(period, gain, q),
            FilterTypeDynamic::Notch => Notch.compute_params(period, gain, q),
            FilterTypeDynamic::AllPass => AllPass.compute_params(period, gain, q),
        }
    }
}
