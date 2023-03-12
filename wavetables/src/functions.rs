pub fn negative(f: impl Fn(f64) -> f64) -> impl Fn(f64) -> f64 {
    move |t| -f(t)
}

pub fn reversed(f: impl Fn(f64) -> f64) -> impl Fn(f64) -> f64 {
    move |t| f(1.0 - t)
}

pub fn join(f1: impl Fn(f64) -> f64, f2: impl Fn(f64) -> f64) -> impl Fn(f64) -> f64 {
    move |t| {
        if t < 0.5 {
            f1(t * 2.0)
        } else {
            f2((t - 0.5) * 2.0)
        }
    }
}

pub fn shift(shift: f64, f: impl Fn(f64) -> f64) -> impl Fn(f64) -> f64 {
    move |t| f((t + shift).rem_euclid(1.0))
}

pub fn scale(scale: f64, f: impl Fn(f64) -> f64) -> impl Fn(f64) -> f64 {
    move |t| f((t * scale).rem_euclid(1.0))
}

pub fn blend(r: f64, f1: impl Fn(f64) -> f64, f2: impl Fn(f64) -> f64) -> impl Fn(f64) -> f64 {
    move |t| f1(t) * (1.0 - r) + f2(t) * r
}

pub fn dynamic_blend(
    f: impl Fn(f64) -> f64,
    f1: impl Fn(f64) -> f64,
    f2: impl Fn(f64) -> f64,
) -> impl Fn(f64) -> f64 {
    move |t| {
        let r = f(t);
        f1(t) * (1.0 - r) + f2(t) * r
    }
}

pub fn product(f1: impl Fn(f64) -> f64, f2: impl Fn(f64) -> f64) -> impl Fn(f64) -> f64 {
    move |t| f2(f1(t) * 0.5 + 0.5)
}

pub fn mul(f1: impl Fn(f64) -> f64, f2: impl Fn(f64) -> f64) -> impl Fn(f64) -> f64 {
    move |t| f1(t) * f2(t)
}
