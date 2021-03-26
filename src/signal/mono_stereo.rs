use super::{C1f64, C2f64, Signal};


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
    type Stereo = C2f64;

    #[inline]
    fn from_m(m: f64) -> Self {
        m
    }

    #[inline]
    fn get_m(&self) -> f64 {
        *self
    }
}

impl Stereo for C2f64 {
    type Mono = C1f64;

    #[inline]
    fn from_lr(l: f64, r: f64) -> Self {
        C2f64([l, r])
    }

    #[inline]
    fn get_l(&self) -> f64 {
        self.0[0]
    }

    #[inline]
    fn get_r(&self) -> f64 {
        self.0[1]
    }
}
