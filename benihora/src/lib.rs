mod benihora;
mod glottis;
mod interval_timer;
pub mod managed;
mod noise;
mod tract;

pub use self::benihora::Benihora;
pub use glottis::Glottis;
pub use interval_timer::IntervalTimer;
pub use managed::BenihoraManaged;
pub use tract::{Constriction, Mouth, Nose};

type F = f64;

pub fn simplex1(x: F) -> F {
    perlin_noise::perlin_noise([x * 1.2, -x * 0.7, 0.0])
}

#[inline]
pub fn lerp(a: F, b: F, t: F) -> F {
    a + (b - a) * t
}
