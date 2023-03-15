pub mod float_array;
pub mod float_array_signal;
pub mod into_mono;
pub mod into_stereo;
pub mod mono_stereo;

pub use into_mono::IntoMono;
pub use into_stereo::IntoStereo;
pub use mono_stereo::{Mono, Stereo};
use num_traits::{Float, NumOps, One, Zero};

use self::float_array::FloatArray;

pub type StereoF64 = FloatArray<2, f64>;

pub trait Signal:
    Default
    + Copy
    + NumOps
    + NumOps<Self::Float>
    + From<Self::Float>
    + Zero
    + One
    + Send
    + Sync
    + 'static
{
    type Float: Float;
    const CHANNEL: usize;

    fn map(self, f: impl Fn(Self::Float) -> Self::Float) -> Self;
    fn zip_map(self, other: Self, f: impl Fn(Self::Float, Self::Float) -> Self::Float) -> Self;
    fn float_from_f64(x: f64) -> Self::Float;

    fn floor(self) -> Self {
        self.map(|x| x.floor())
    }

    fn ceil(self) -> Self {
        self.map(|x| x.ceil())
    }

    fn round(self) -> Self {
        self.map(|x| x.round())
    }

    fn trunc(self) -> Self {
        self.map(|x| x.trunc())
    }

    fn fract(self) -> Self {
        self.map(|x| x.fract())
    }

    fn abs(self) -> Self {
        self.map(|x| x.abs())
    }

    fn mul_add(self, a: Self::Float, b: Self::Float) -> Self {
        self.map(|x| x.mul_add(a, b))
    }

    fn recip(self) -> Self {
        self.map(|x| x.recip())
    }

    fn powi(self, n: i32) -> Self {
        self.map(|x| x.powi(n))
    }

    fn powf(self, n: Self::Float) -> Self {
        self.map(|x| x.powf(n))
    }

    fn sqrt(self) -> Self {
        self.map(|x| x.sqrt())
    }

    fn exp(self) -> Self {
        self.map(|x| x.exp())
    }

    fn exp2(self) -> Self {
        self.map(|x| x.exp2())
    }

    fn ln(self) -> Self {
        self.map(|x| x.ln())
    }

    fn log(self, base: Self::Float) -> Self {
        self.map(|x| x.log(base))
    }

    fn log2(self) -> Self {
        self.map(|x| x.log2())
    }

    fn log10(self) -> Self {
        self.map(|x| x.log10())
    }

    fn max(self, other: Self) -> Self {
        self.zip_map(other, |x, y| x.max(y))
    }

    fn min(self, other: Self) -> Self {
        self.zip_map(other, |x, y| x.min(y))
    }

    fn cbrt(self) -> Self {
        self.map(|x| x.cbrt())
    }

    fn sin(self) -> Self {
        self.map(|x| x.sin())
    }

    fn cos(self) -> Self {
        self.map(|x| x.cos())
    }

    fn tan(self) -> Self {
        self.map(|x| x.tan())
    }

    fn asin(self) -> Self {
        self.map(|x| x.asin())
    }

    fn acos(self) -> Self {
        self.map(|x| x.acos())
    }

    fn atan(self) -> Self {
        self.map(|x| x.atan())
    }

    // fn sin_cos(self) -> (Self, Self) {
    //     self.map(|x| {let (s, c) = x.sin_cos(); (s, c)})});
    // }

    fn exp_m1(self) -> Self {
        self.map(|x| x.exp_m1())
    }

    fn ln_1p(self) -> Self {
        self.map(|x| x.ln_1p())
    }

    fn sinh(self) -> Self {
        self.map(|x| x.sinh())
    }

    fn cosh(self) -> Self {
        self.map(|x| x.cosh())
    }

    fn tanh(self) -> Self {
        self.map(|x| x.tanh())
    }

    fn asinh(self) -> Self {
        self.map(|x| x.asinh())
    }

    fn acosh(self) -> Self {
        self.map(|x| x.acosh())
    }

    fn atanh(self) -> Self {
        self.map(|x| x.atanh())
    }
}

impl Signal for f64 {
    type Float = f64;
    const CHANNEL: usize = 1;

    #[inline]
    fn map(self, f: impl Fn(Self::Float) -> Self::Float) -> Self {
        f(self)
    }

    #[inline]
    fn zip_map(self, other: Self, f: impl Fn(Self::Float, Self::Float) -> Self::Float) -> Self {
        f(self, other)
    }

    #[inline]
    fn float_from_f64(x: f64) -> Self::Float {
        x
    }
}
