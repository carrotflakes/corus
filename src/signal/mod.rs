mod into_mono;
mod into_stereo;
mod mono_stereo;

pub use into_mono::IntoMono;
pub use into_stereo::IntoStereo;
pub use mono_stereo::{Mono, Stereo};

use std::ops::{Add, Div, Mul, Neg};

pub trait Signal:
    'static
    + Sized
    + Clone
    + Add<Output = Self>
    + Mul<Output = Self>
    + Mul<f64, Output = Self>
    + Div<f64, Output = Self>
    + Neg<Output = Self>
    + Default
{
    type Float: 'static;

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

impl Mul<f64> for C2f64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        Self([self.0[0] * rhs, self.0[1] * rhs])
    }
}

impl Div<f64> for C2f64 {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        Self([self.0[0] / rhs, self.0[1] / rhs])
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
    fn mul_identity() -> Self {
        Self([1.0, 1.0])
    }

    #[inline]
    fn lerp(&self, other: &Self, r: Self::Float) -> Self {
        self.clone() * (1.0 - r) + other.clone() * r
    }
}
