mod into_mono;
mod into_stereo;

pub use into_mono::IntoMono;
pub use into_stereo::IntoStereo;

use std::ops::{Add, Mul, Neg};

pub trait Signal: 'static + Sized + Clone + Add<Output = Self> + Mul<Output = Self> + Neg<Output = Self> {
    type Float;

    fn get(&self, channel: usize) -> Self::Float;
    fn map<F: Fn(Self::Float) -> Self::Float>(&self, f: F) -> Self;
    fn map2_1<F: Fn(Self::Float, Self::Float) -> Self::Float>(&self, self2: Self, f: F) -> Self;
    fn mul_identity() -> Self;
    fn lerp(&self, other: &Self, r: Self::Float) -> Self;
}

pub type C1f64 = f64;

#[derive(Debug, Clone, Copy, Default)]
pub struct C2f64(pub [f64; 2]);

impl Add for C2f64 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self([self.0[0] + rhs.0[0], self.0[1] + rhs.0[1]])
    }
}

impl Mul for C2f64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self([self.0[0] * rhs.0[0], self.0[1] * rhs.0[1]])
    }
}

impl Neg for C2f64 {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        Self([-self.0[0], -self.0[1]])
    }
}

impl Mul<C1f64> for C2f64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: C1f64) -> Self::Output {
        Self([self.0[0] * rhs.get_m(), self.0[1] * rhs.get_m()])
    }
}

impl From<[f64; 2]> for C2f64 {
    fn from(slice: [f64; 2]) -> Self {
        C2f64(slice)
    }
}

impl Signal for C1f64 {
    type Float = f64;

    #[inline]
    fn get(&self, _channel: usize) -> Self::Float {
        self.get_m()
    }

    #[inline]
    fn map<F: Fn(f64) -> f64>(&self, f: F) -> Self {
        f(self.get_m())
    }

    #[inline]
    fn map2_1<F: Fn(f64, f64) -> f64>(&self, self2: Self, f: F) -> Self {
        f(self.get_m(), self2.get_m())
    }

    #[inline]
    fn mul_identity() -> Self {
        1.0
    }

    #[inline]
    fn lerp(&self, other: &Self, r: Self::Float) -> Self {
        self * (1.0 - r) + other * r
    }
}

impl Signal for C2f64 {
    type Float = f64;

    #[inline]
    fn get(&self, channel: usize) -> Self::Float {
        self.0[channel]
    }

    #[inline]
    fn map<F: Fn(f64) -> f64>(&self, f: F) -> Self {
        Self([f(self.0[0]), f(self.0[1])])
    }

    #[inline]
    fn map2_1<F: Fn(f64, f64) -> f64>(&self, self2: Self, f: F) -> Self {
        Self([f(self.0[0], self2.0[0]), f(self.0[1], self2.0[1])])
    }

    #[inline]
    fn mul_identity() -> Self {
        Self([1.0, 1.0])
    }

    #[inline]
    fn lerp(&self, other: &Self, r: Self::Float) -> Self {
        self.clone() * (1.0 - r) + other.clone() * r
    }
}

pub trait Mono<F>: Signal<Float = F> + Mul<Output = Self> + Add<Output = Self> {
    type Stereo: Stereo<F>;

    fn from_m(m: F) -> Self;
    fn get_m(&self) -> F;
}

pub trait Stereo<F>: Signal<Float = F> {
    type Mono: Mono<F>;

    fn from_lr(l: F, r: F) -> Self;
    fn get_l(&self) -> F;
    fn get_r(&self) -> F;
}

impl Signal for f32 {
    type Float = f32;

    #[inline]
    fn get(&self, _channel: usize) -> Self::Float {
        *self
    }

    #[inline]
    fn map<F: Fn(f32) -> f32>(&self, f: F) -> Self {
        f(*self)
    }

    #[inline]
    fn map2_1<F: Fn(f32, f32) -> f32>(&self, self2: Self, f: F) -> Self {
        f(*self, self2)
    }

    #[inline]
    fn mul_identity() -> Self {
        1.0
    }

    #[inline]
    fn lerp(&self, other: &Self, r: Self::Float) -> Self {
        self * (1.0 - r) + other * r
    }
}

impl Mono<f64> for f64 {
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

impl Stereo<f64> for C2f64 {
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
