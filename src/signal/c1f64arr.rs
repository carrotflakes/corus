use super::Signal;

use std::ops::{Add, Div, Mul, Neg};

#[derive(Debug, Clone, Copy)]
pub struct C1f64Arr<const N: usize>(pub [f64; N]);

impl<const N: usize> Add for C1f64Arr<N> {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let mut arr = [Default::default(); N];
        for i in 0..N {
            arr[i] = self.0[i] + rhs.0[i];
        }
        Self(arr)
    }
}

impl<const N: usize> Mul for C1f64Arr<N> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let mut arr = [Default::default(); N];
        for i in 0..N {
            arr[i] = self.0[i] * rhs.0[i];
        }
        Self(arr)
    }
}

impl<const N: usize> Neg for C1f64Arr<N> {
    type Output = Self;

    #[inline]
    fn neg(self) -> Self::Output {
        let mut arr = [Default::default(); N];
        for i in 0..N {
            arr[i] = -self.0[i];
        }
        Self(arr)
    }
}

impl<const N: usize> Mul<f64> for C1f64Arr<N> {
    type Output = Self;

    #[inline]
    fn mul(self, rhs: f64) -> Self::Output {
        let mut arr = [Default::default(); N];
        for i in 0..N {
            arr[i] = self.0[i] * rhs;
        }
        Self(arr)
    }
}

impl<const N: usize> Div<f64> for C1f64Arr<N> {
    type Output = Self;

    #[inline]
    fn div(self, rhs: f64) -> Self::Output {
        let mut arr = [Default::default(); N];
        for i in 0..N {
            arr[i] = self.0[i] / rhs;
        }
        Self(arr)
    }
}

impl<const N: usize> Default for C1f64Arr<N> {
    #[inline]
    fn default() -> Self {
        C1f64Arr([0.0; N])
    }
}

impl<const N: usize> Signal for C1f64Arr<N> {
    type Float = f64;

    fn mul_identity() -> Self {
        C1f64Arr([1.0; N])
    }

    fn lerp(&self, other: &Self, r: Self::Float) -> Self {
        let mut arr = [0.0; N];
        for i in 0..N {
            arr[i] = self.0[i] * (1.0 - r) + other.0[i] * r;
        }
        Self(arr)
    }
}
