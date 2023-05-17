use std::f64::consts::TAU;

pub fn unipolar_sin_pulse(width: f64, t: f64) -> f64 {
    if t < width {
        0.5 - (t * TAU / width).cos() * 0.5
    } else {
        0.0
    }
}
