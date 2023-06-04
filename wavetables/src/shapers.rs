use std::f64::consts::FRAC_PI_2;

pub fn linear(x: f64) -> f64 {
    x
}

pub fn hard_clip(x: f64) -> f64 {
    x.clamp(-1.0, 1.0)
}

pub fn tanh(x: f64) -> f64 {
    x.tanh()
}

pub fn sin(x: f64) -> f64 {
    (x * FRAC_PI_2).sin()
}

pub fn wrap(x: f64) -> f64 {
    (x + 1.0).rem_euclid(2.0) - 1.0
}

pub fn triangle(x: f64) -> f64 {
    let x = (x + 1.0).rem_euclid(4.0);
    if x < 2.0 {
        x - 1.0
    } else {
        3.0 - x
    }
}
