mod benihora;
mod glottis;
mod tract;

pub use benihora::Benihora;
pub use glottis::Glottis;
pub use tract::{Constriction, Mouth, Nose, Tract};

type F = f64;

fn simplex1(x: F) -> F {
    perlin_noise::perlin_noise([x * 1.2, -x * 0.7, 0.0])
}

#[inline]
fn lerp(a: F, b: F, t: F) -> F {
    a + (b - a) * t
}
