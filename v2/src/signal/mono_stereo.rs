use super::{Signal, StereoF64};

pub trait Mono: Signal {
    type Stereo: Stereo<Float = Self::Float>;

    fn from_m(m: Self::Float) -> Self;
    fn get_m(&self) -> Self::Float;
}

pub trait Stereo: Signal {
    type Mono: Mono<Float = Self::Float>;

    fn from_lr(l: Self::Float, r: Self::Float) -> Self;
    fn get_l(&self) -> Self::Float;
    fn get_r(&self) -> Self::Float;
}

impl Mono for f64 {
    type Stereo = StereoF64;

    #[inline]
    fn from_m(m: f64) -> Self {
        m
    }

    #[inline]
    fn get_m(&self) -> f64 {
        *self
    }
}

impl Stereo for StereoF64 {
    type Mono = f64;

    #[inline]
    fn from_lr(l: f64, r: f64) -> Self {
        [l, r]
    }

    #[inline]
    fn get_l(&self) -> f64 {
        self[0]
    }

    #[inline]
    fn get_r(&self) -> f64 {
        self[1]
    }
}
