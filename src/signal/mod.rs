mod into_stereo;

pub use into_stereo::IntoStereo;

use std::ops::{Add, Mul};

pub trait Signal: 'static + Sized {
    type Float;

    fn get(&self, channel: usize) -> Self::Float;
    fn map<F: Fn(Self::Float) -> Self::Float>(&self, f: F) -> Self;
    fn map2_1<F: Fn(Self::Float, Self::Float) -> Self::Float>(&self, self2: Self, f: F) -> Self;
}


#[derive(Debug, Clone, Copy, Default)]
pub struct C1f32(pub [f32; 1]);

#[derive(Debug, Clone, Copy, Default)]
pub struct C1f64(pub [f64; 1]);

#[derive(Debug, Clone, Copy, Default)]
pub struct C2f32(pub [f32; 2]);

#[derive(Debug, Clone, Copy, Default)]
pub struct C2f64(pub [f64; 2]);


impl Add for C1f32 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self([self.0[0] + rhs.0[0]])
    }
}

impl Add for C1f64 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self([self.0[0] + rhs.0[0]])
    }
}

impl Add for C2f32 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self([self.0[0] + rhs.0[0], self.0[1] + rhs.0[1]])
    }
}

impl Add for C2f64 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self([self.0[0] + rhs.0[0], self.0[1] + rhs.0[1]])
    }
}


impl Mul for C1f32 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self([self.0[0] * rhs.0[0]])
    }
}

impl Mul for C1f64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self([self.0[0] * rhs.0[0]])
    }
}

impl Mul for C2f32 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self([self.0[0] * rhs.0[0], self.0[1] * rhs.0[1]])
    }
}

impl Mul for C2f64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self([self.0[0] * rhs.0[0], self.0[1] * rhs.0[1]])
    }
}


impl Mul<C1f32> for C2f32 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: C1f32) -> Self::Output {
        Self([self.0[0] * rhs.0[0], self.0[1] * rhs.0[0]])
    }
}

impl Mul<C1f64> for C2f64 {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: C1f64) -> Self::Output {
        Self([self.0[0] * rhs.0[0], self.0[1] * rhs.0[0]])
    }
}


impl From<f32> for C1f32 {
    #[inline]
    fn from(v: f32) -> Self {
        C1f32([v])
    }
}

impl From<f64> for C1f64 {
    #[inline]
    fn from(v: f64) -> Self {
        C1f64([v])
    }
}


impl Signal for C1f32 {
    type Float = f32;

    #[inline]
    fn get(&self, channel: usize) -> Self::Float {
        self.0[channel]
    }

    #[inline]
    fn map<F: Fn(f32) -> f32>(&self, f: F) -> Self {
        Self([f(self.0[0])])
    }

    #[inline]
    fn map2_1<F: Fn(f32, f32) -> f32>(&self, self2: Self, f: F) -> Self {
        Self([f(self.0[0], self2.0[0])])
    }
}

impl Signal for C2f32 {
    type Float = f32;

    #[inline]
    fn get(&self, channel: usize) -> Self::Float {
        self.0[channel]
    }

    #[inline]
    fn map<F: Fn(f32) -> f32>(&self, f: F) -> Self {
        Self([f(self.0[0]), f(self.0[1])])
    }

    #[inline]
    fn map2_1<F: Fn(f32, f32) -> f32>(&self, self2: Self, f: F) -> Self {
        Self([f(self.0[0], self2.0[0]), f(self.0[1], self2.0[1])])
    }
}

impl Signal for C1f64 {
    type Float = f64;

    #[inline]
    fn get(&self, channel: usize) -> Self::Float {
        self.0[channel]
    }

    #[inline]
    fn map<F: Fn(f64) -> f64>(&self, f: F) -> Self {
        Self([f(self.0[0])])
    }

    #[inline]
    fn map2_1<F: Fn(f64, f64) -> f64>(&self, self2: Self, f: F) -> Self {
        Self([f(self.0[0], self2.0[0])])
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
}
