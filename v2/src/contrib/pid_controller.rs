use crate::{signal::Signal, ProcessContext};

/// kp: proportional gain 0.0 ~
/// ki: integral gain 0.0 ~
/// kd: derivative gain 0.0 ~ 1.0
pub struct PIDController<S: Signal> {
    pub kp: S::Float,
    pub ki: S::Float,
    pub kd: S::Float,
    pub integral: S,
    pub last: Option<S>,
}

impl<S: Signal> PIDController<S> {
    pub fn new(kp: S::Float, ki: S::Float, kd: S::Float) -> Self {
        Self {
            kp,
            ki,
            kd,
            integral: S::default(),
            last: None,
        }
    }

    pub fn reset(&mut self) {
        self.integral = S::default();
        self.last = None;
    }

    pub fn process(&mut self, ctx: &ProcessContext, x: S) -> S {
        let d = self.last.map(|last| (x - last) * S::float_from_f64(ctx.sample_rate())).unwrap_or_default();
        self.integral = self.integral + x * S::float_from_f64(ctx.dtime());
        let y = x * self.kp + self.integral * self.ki - d * self.kd;
        self.last = Some(x);
        y
    }
}
