mod benihora;
pub mod glottis;
mod interval_timer;
pub mod managed;
mod noise;
pub mod resample;
pub mod tract;
pub mod wiggle;

pub use self::benihora::Benihora;
pub use glottis::Glottis;
pub use interval_timer::IntervalTimer;
pub use managed::BenihoraManaged;
pub use tract::Constriction;

type F = f64;

#[inline]
pub fn lerp(a: F, b: F, t: F) -> F {
    a + (b - a) * t
}

pub fn rand_f64(seed: &mut u32) -> f64 {
    *seed = seed.overflowing_mul(48271).0 % ((1 << 31) - 1);
    (*seed << 1) as f64 / std::u32::MAX as f64
}
