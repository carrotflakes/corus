use crate::{signal::Mono, Event};

use super::{Node, ProcContext};

#[derive(Clone)]
pub enum ParamState<F: Float> {
    Constant,
    Linear(F),
    Exponential(F),
    Target { target: F, time_constant: f64 },
}

pub struct Param<F: Float> {
    value: F,
    state: ParamState<F>,
}

impl<F: Float> Param<F> {
    pub fn new() -> Self {
        Self::with_value(Default::default())
    }

    pub fn with_value(value: F) -> Self {
        Param {
            value,
            state: ParamState::Constant,
        }
    }
}

impl Node<f64> for Param<f64> {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> f64 {
        let value = self.value.clone();
        match self.state {
            ParamState::Constant => {}
            ParamState::Linear(v) => {
                self.value = self.value + v / ctx.sample_rate as f64;
            }
            ParamState::Exponential(v) => {
                self.value = self.value * v.powf(1.0 / ctx.sample_rate as f64);
            }
            ParamState::Target {
                target,
                time_constant,
            } => {
                self.value = (self.value - target)
                    / (1.0 / (time_constant * ctx.sample_rate as f64)).exp()
                    + target;
            }
        }
        f64::from_m(value)
    }

    fn lock(&mut self) {}

    fn unlock(&mut self) {}
}

pub trait Float: 'static + Clone + Default + From<f64> + Into<f64> {}

impl Float for f64 {}

#[derive(Debug, Clone)]
pub enum ParamEvent<F: Float> {
    SetValueAtTime { value: F },
    LinearRampToValueAtTime { value: F },
    ExponentialRampToValueAtTime { value: F },
    SetTargetAtTime { target: F, time_constant: f64 },
}

impl<F: Float> Event for ParamEvent<F> {
    type Target = Param<F>;

    fn dispatch(&self, _time: f64, target: &mut Self::Target) {
        match self {
            ParamEvent::SetValueAtTime { value } => {
                target.value = value.clone();
                target.state = ParamState::Constant;
            }
            ParamEvent::LinearRampToValueAtTime { value } => {
                target.state = ParamState::Linear(value.clone());
            }
            ParamEvent::ExponentialRampToValueAtTime { value } => {
                target.state = ParamState::Exponential(value.clone());
            }
            ParamEvent::SetTargetAtTime {
                target: t,
                time_constant,
            } => {
                target.state = ParamState::Target {
                    target: t.clone(),
                    time_constant: *time_constant,
                };
            }
        }
    }
}

#[test]
fn test() {
    let mut param = crate::EventControlInplace::new(Param::<f64>::new());
    let mut pc = ProcContext::new(4);

    param.push_event(2.0 / 4.0, ParamEvent::SetValueAtTime { value: 1.0 });
    param.push_event(4.0 / 4.0, ParamEvent::SetValueAtTime { value: 2.0 });
    param.push_event(
        4.0 / 4.0,
        ParamEvent::LinearRampToValueAtTime { value: -2.0 * 4.0 },
    );
    param.push_event(
        6.0 / 4.0,
        ParamEvent::LinearRampToValueAtTime {
            value: 3.0 / 4.0 * 4.0,
        },
    );
    param.push_event(10.0 / 4.0, ParamEvent::SetValueAtTime { value: 1.0 });
    param.push_event(
        12.0 / 4.0,
        ParamEvent::SetTargetAtTime {
            target: 0.0,
            time_constant: 0.5,
        },
    );
    param.push_event(15.0 / 4.0, ParamEvent::SetValueAtTime { value: 0.1 });
    // param.set_value_at_time(2.0 / 4.0, 1.0);
    // param.set_value_at_time(4.0 / 4.0, 2.0);
    // param.linear_ramp_to_value_at_time(6.0 / 4.0, -2.0);
    // param.linear_ramp_to_value_at_time(10.0 / 4.0, 1.0);
    // param.set_target_at_time(12.0 / 4.0, 0.0, 0.5);
    // param.cancel_and_hold_at_time(15.0 / 4.0);
    // param.exponential_ramp_to_value_at_time(19.0 / 4.0, 1.0);

    for _ in 0..20 {
        dbg!(pc.time);
        dbg!(param.proc(&pc));
        pc.time += 1.0 / pc.sample_rate as f64;
    }
}
