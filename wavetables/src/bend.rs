use std::f64::consts::PI;

/// level: -1.0 ~ 1.0 (positive: bend minus, negative: bend plus, 0.0: no bend)
pub fn quadratic_bender(level: f64) -> impl Fn(f64) -> f64 {
    move |t| t.powi(2) * level + t * (1.0 - level)
}

/// level: -0.5 ~ 4.0
pub fn cubic_bender(level: f64) -> impl Fn(f64) -> f64 {
    move |t| {
        let t = 2.0 * t - 1.0;
        let t = t.powi(3) * level + t * (1.0 - level);
        t * 0.5 + 0.5
    }
}

/// level: -1.0 ~ 1.0 (positive: bend plus, negative: bend minus, 0.0: no bend)
pub fn sin_bender(level: f64) -> impl Fn(f64) -> f64 {
    move |t| t + (t * PI).sin() * level / PI
}

/// level: 0.0 ~ 1.0
pub fn cos_bender(level: f64) -> impl Fn(f64) -> f64 {
    move |t| {
        let a = 0.5 - (t * PI).cos() * 0.5;
        a * level + t * (1.0 - level)
    }
}
