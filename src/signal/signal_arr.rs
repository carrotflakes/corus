use super::Signal;

use std::ops::{Add, Div, Mul, Neg};

#[derive(Debug, Clone, Copy)]
pub struct SignalArr<S: Signal, const N: usize>(pub [S; N]);

impl<S: Signal, const N: usize> SignalArr<S, N> {
    #[inline]
    pub fn new(arr: [S; N]) -> Self {
        SignalArr(arr)
    }
}

impl<S: Signal, const N: usize> Add for SignalArr<S, N> {
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

impl<S: Signal, const N: usize> Mul for SignalArr<S, N> {
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

impl<S: Signal, const N: usize> Neg for SignalArr<S, N> {
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

impl<S: Signal, const N: usize> Mul<f64> for SignalArr<S, N> {
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

impl<S: Signal, const N: usize> Div<f64> for SignalArr<S, N> {
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

impl<S: Signal, const N: usize> Default for SignalArr<S, N> {
    #[inline]
    fn default() -> Self {
        SignalArr([Default::default(); N])
    }
}

// impl<S: Signal<Float = f64>, const N: usize> Signal for SignalArr<S, N> {
//     type Float = [f64; N];

//     #[inline]
//     fn get(&self, channel: usize) -> Self::Float {
//         let mut arr = [0.0; N];
//         for i in 0..N {
//             arr[i] = self.0[i].get(channel);
//         }
//         arr
//     }

//     #[inline]
//     fn map<F: Fn([f64; N]) -> [f64; N]>(&self, f: F) -> Self {
//         let mut arr = [0.0; N];
//         for i in 0..N {
//             arr[i] = self.0[i];
//         }
//         // f(self.get_m())
//     }

//     #[inline]
//     fn map2_1<F: Fn([f64; N], [f64; N]) -> [f64; N]>(&self, self2: Self, f: F) -> Self {
//         f(self.get_m(), self2.get_m())
//     }

//     #[inline]
//     fn mul_identity() -> Self {
//         1.0
//     }

//     #[inline]
//     fn lerp(&self, other: &Self, r: Self::Float) -> Self {
//         self * (1.0 - r) + other * r
//     }
// }
