use std::f64::consts::TAU;

pub fn sin(t: f64) -> f64 {
    (t * TAU).sin()
}

pub fn triangle(t: f64) -> f64 {
    if t < 0.25 {
        4.0 * t
    } else if t < 0.75 {
        2.0 - 4.0 * t
    } else {
        4.0 * t - 4.0
    }
}

pub fn shifted_triangle(t: f64) -> f64 {
    if t < 0.5 {
        4.0 * t - 1.0
    } else {
        3.0 - 4.0 * t
    }
}

pub fn saw(t: f64) -> f64 {
    2.0 * t - 1.0
}

pub fn square(t: f64) -> f64 {
    if t < 0.5 {
        -1.0
    } else {
        1.0
    }
}

pub fn pulse(width: f64, t: f64) -> f64 {
    if t < width {
        -1.0
    } else {
        1.0
    }
}

pub fn quadratic(t: f64) -> f64 {
    t * t * 2.0 - 1.0
}
