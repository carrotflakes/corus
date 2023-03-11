pub mod into_mono;
pub mod into_stereo;
pub mod mono_stereo;

pub use into_mono::IntoMono;
pub use into_stereo::IntoStereo;
pub use mono_stereo::{Mono, Stereo};

pub type StereoF64 = [f64; 2];

pub trait Signal {
    type Float;
}

impl Signal for f64 {
    type Float = f64;
}

impl Signal for StereoF64 {
    type Float = f64;
}
