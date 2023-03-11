pub mod into_mono;
pub mod into_stereo;
pub mod mono_stereo;

pub use into_mono::IntoMono;
pub use into_stereo::IntoStereo;
pub use mono_stereo::{Mono, Stereo};
use num_traits::Float;

pub type StereoF64 = [f64; 2];

pub trait Signal: Default + Copy + Send + Sync + 'static {
    type Float: Float;
    const CHANNEL: usize;
}

impl Signal for f64 {
    type Float = f64;
    const CHANNEL: usize = 1;
}

impl Signal for StereoF64 {
    type Float = f64;
    const CHANNEL: usize = 2;
}

pub trait SignalExt: Signal {
    fn add(self, rhs: Self) -> Self;
    fn mul(self, rhs: Self) -> Self;
    fn from_float(f: Self::Float) -> Self;
}

impl SignalExt for f64 {
    fn add(self, rhs: Self) -> Self {
        self + rhs
    }

    fn mul(self, rhs: Self) -> Self {
        self * rhs
    }

    fn from_float(f: Self::Float) -> Self {
        f
    }
}

impl SignalExt for StereoF64 {
    fn add(self, rhs: Self) -> Self {
        [self[0] + rhs[0], self[1] + rhs[1]]
    }

    fn mul(self, rhs: Self) -> Self {
        [self[0] * rhs[0], self[1] * rhs[1]]
    }

    fn from_float(f: Self::Float) -> Self {
        [f, f]
    }
}
