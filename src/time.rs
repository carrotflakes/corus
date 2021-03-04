#[derive(Debug, Clone, Copy)]
pub struct Sample(pub u64);

#[derive(Debug, Clone, Copy)]
pub struct Second(pub f64);

pub trait AsSample {
    fn as_sample(&self, sample_rate: u64) -> u64;
}

pub trait AsSecond {
    fn as_second(&self, sample_rate: u64) -> f64;
}

impl AsSample for Sample {
    #[inline]
    fn as_sample(&self, _sample_rate: u64) -> u64 {
        self.0
    }
}

impl AsSecond for Sample {
    #[inline]
    fn as_second(&self, sample_rate: u64) -> f64 {
        self.0 as f64 / sample_rate as f64
    }
}

impl AsSample for Second {
    #[inline]
    fn as_sample(&self, sample_rate: u64) -> u64 {
        (self.0 * sample_rate as f64) as u64
    }
}

impl AsSecond for Second {
    #[inline]
    fn as_second(&self, _sample_rate: u64) -> f64 {
        self.0
    }
}