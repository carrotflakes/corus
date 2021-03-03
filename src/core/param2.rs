use std::{
    collections::VecDeque,
    ops::{Div, Sub},
};

use crate::{signal::Mono, Event, EventControl};

use super::{Node, ProcContext};

#[derive(Clone)]
pub enum ParamState<F: Float> {
    Constant(F),
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
            value: value.clone(),
            state: ParamState::Constant(value),
        }
    }
}

impl Node<f64> for Param<f64> {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> f64 {
        let value = self.value.clone();
        match self.state {
            ParamState::Constant(_) => {}
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

pub trait Float:
    'static + Clone + Default + From<f64> + Into<f64> + Div<Output = Self> + Sub<Output = Self>
{
    fn linear_interpolate(&self, other: Self, r: f64) -> Self;
    fn exponential_interpolate(&self, other: Self, r: f64) -> Self;
    fn powf(&self, other: Self) -> Self;
}

impl Float for f64 {
    #[inline]
    fn linear_interpolate(&self, other: Self, r: f64) -> Self {
        self * (1.0 - r) + other * r
    }

    #[inline]
    fn exponential_interpolate(&self, other: Self, r: f64) -> Self {
        (self.ln() * (1.0 - r) + other.ln() * r).exp()
    }

    #[inline]
    fn powf(&self, other: Self) -> Self {
        f64::powf(*self, other)
    }
}

impl<F: Float> Event for ParamState<F> {
    type Target = Param<F>;

    fn dispatch(&self, _time: f64, target: &mut Param<F>) {
        match self {
            ParamState::Constant(value) => {
                target.value = value.clone();
            }
            _ => {}
        }
        target.state = self.clone();
    }
}

#[derive(Debug, Clone)]
pub enum ParamEvent<F: Float> {
    SetValueAtTime { value: F },
    LinearRampToValueAtTime { value: F },
    ExponentialRampToValueAtTime { value: F },
    SetTargetAtTime { target: F, time_constant: f64 },
}

pub struct ParamEventSchedule<F: Float> {
    events: VecDeque<(f64, ParamEvent<F>)>,
    last_event: (f64, ParamEvent<F>),
    control: EventControl<ParamState<F>>,
}

impl<F: Float> ParamEventSchedule<F> {
    pub fn new(control: EventControl<ParamState<F>>) -> Self {
        Self {
            events: VecDeque::new(),
            last_event: (
                0.0,
                ParamEvent::SetValueAtTime {
                    value: F::from(0.0),
                },
            ),
            control,
        }
    }

    pub fn push_event(&mut self, time: f64, event: ParamEvent<F>) {
        for (i, e) in self.events.iter().enumerate() {
            if time < e.0 {
                self.events.insert(i, (time, event));
                return;
            }
        }
        self.events.push_back((time, event));
    }

    pub fn set_value_at_time(&mut self, time: f64, value: F) {
        self.push_event(time, ParamEvent::SetValueAtTime { value });
    }

    pub fn linear_ramp_to_value_at_time(&mut self, time: f64, value: F) {
        self.push_event(time, ParamEvent::LinearRampToValueAtTime { value });
    }

    pub fn exponential_ramp_to_value_at_time(&mut self, time: f64, value: F) {
        self.push_event(time, ParamEvent::ExponentialRampToValueAtTime { value });
    }

    pub fn set_target_at_time(&mut self, time: f64, target: F, time_constant: f64) {
        self.push_event(
            time,
            ParamEvent::SetTargetAtTime {
                target,
                time_constant,
            },
        );
    }

    fn cancel_scheduled_values_(&mut self, time: f64) -> Option<(f64, ParamEvent<F>)> {
        if let Some(i) = self.events.iter().position(|e| time <= e.0) {
            let e = self.events[i].clone();
            self.events.truncate(i);
            Some(e)
        } else {
            None
        }
    }

    pub fn cancel_scheduled_values(&mut self, time: f64) {
        self.cancel_scheduled_values_(time);
    }

    pub fn cancel_and_hold_at_time(&mut self, time: f64) {
        let value = self.compute_value(time);
        if let Some(e) = self.cancel_scheduled_values_(time) {
            match e.1 {
                ParamEvent::SetValueAtTime { .. } | ParamEvent::SetTargetAtTime { .. } => {
                    self.set_value_at_time(time, value)
                }
                ParamEvent::LinearRampToValueAtTime { .. } => {
                    self.linear_ramp_to_value_at_time(time, value)
                }
                ParamEvent::ExponentialRampToValueAtTime { .. } => {
                    self.exponential_ramp_to_value_at_time(time, value)
                }
            }
        } else {
            self.set_value_at_time(time, value); // OK?
        }
    }

    pub fn compute_value(&self, time: f64) -> F {
        let mut before = Some(&self.last_event);
        let mut after = None;
        for event in &self.events {
            if time < event.0 {
                match event.1 {
                    ParamEvent::SetValueAtTime { .. } => {}
                    ParamEvent::LinearRampToValueAtTime { .. }
                    | ParamEvent::ExponentialRampToValueAtTime { .. } => {
                        after = Some(event);
                    }
                    ParamEvent::SetTargetAtTime { .. } => {}
                }
                break;
            }
            match event.1 {
                ParamEvent::SetValueAtTime { .. } => {
                    before = Some(event);
                    after = None;
                }
                ParamEvent::LinearRampToValueAtTime { .. }
                | ParamEvent::ExponentialRampToValueAtTime { .. } => {
                    before = Some(event);
                    after = None;
                }
                ParamEvent::SetTargetAtTime { .. } => {
                    after = Some(event);
                }
            }
        }
        if let Some(before) = before {
            let before_value = match before.1.clone() {
                ParamEvent::SetValueAtTime { value }
                | ParamEvent::LinearRampToValueAtTime { value }
                | ParamEvent::ExponentialRampToValueAtTime { value } => value,
                ParamEvent::SetTargetAtTime { .. } => {
                    unreachable!()
                }
            };
            if let Some(after) = after {
                match after.1.clone() {
                    ParamEvent::SetValueAtTime { .. } => {
                        unreachable!()
                    }
                    ParamEvent::LinearRampToValueAtTime { value } => {
                        let r = (time - before.0) / (after.0 - before.0);
                        before_value.linear_interpolate(value, r)
                    }
                    ParamEvent::ExponentialRampToValueAtTime { value } => {
                        let r = (time - before.0) / (after.0 - before.0);
                        before_value.exponential_interpolate(value, r)
                    }
                    ParamEvent::SetTargetAtTime {
                        target,
                        time_constant,
                    } => {
                        let t = (time - after.0) as f64;
                        let r = 1.0 - (-t / time_constant).exp();
                        before_value.linear_interpolate(target, r)
                    }
                }
            } else {
                before_value
            }
        } else {
            unreachable!()
        }
    }

    pub fn dispatch(&mut self, time: f64) {
        let mut before_value = self.compute_value(self.last_event.0);
        while !self.events.is_empty() {
            let first = &self.events[0];
            match first.1.clone() {
                ParamEvent::SetValueAtTime { value } => {
                    if first.0 < time {
                        self.control.push(first.0, ParamState::Constant(value.clone()));
                        before_value = value.clone();
                        self.last_event = first.clone();
                    }
                }
                ParamEvent::LinearRampToValueAtTime { value } => {
                    self.control.push(
                        self.last_event.0,
                        ParamState::Linear(
                            (value.clone() - before_value.clone())
                                / F::from(first.0 - self.last_event.0),
                        ),
                    );
                    if first.0 < time {
                        self.control.push(first.0, ParamState::Constant(value.clone()));
                        before_value = value.clone();
                        self.last_event = (first.0, ParamEvent::SetValueAtTime { value });
                    }
                }
                ParamEvent::ExponentialRampToValueAtTime { value } => {
                    self.control.push(
                        self.last_event.0,
                        ParamState::Exponential(
                            (value.clone() / before_value.clone())
                                .powf(F::from(1.0 / (first.0 - self.last_event.0))),
                        ),
                    );
                    if first.0 < time {
                        self.control.push(first.0, ParamState::Constant(value.clone()));
                        before_value = value.clone();
                        self.last_event = (first.0, ParamEvent::SetValueAtTime { value });
                    }
                }
                ParamEvent::SetTargetAtTime {
                    target,
                    time_constant,
                } => {
                    if first.0 < time {
                        self.control.push(
                            first.0,
                            ParamState::Target {
                                target,
                                time_constant,
                            },
                        );
                        self.last_event = first.clone();
                    }
                }
            }
            if time < first.0 {
                // 位置、大丈夫？
                break;
            }
            self.events.pop_front();
        }
    }
}

#[test]
fn test() {
    let mut eq = crate::EventQueue::new();
    let param = std::sync::Arc::new(Param::<f64>::new());
    let mut schedule = ParamEventSchedule::new(eq.get_controller(&param));
    let mut pc = ProcContext::new(4);

    schedule.set_value_at_time(2.0 / 4.0, 1.0);
    schedule.set_value_at_time(4.0 / 4.0, 2.0);
    schedule.linear_ramp_to_value_at_time(6.0 / 4.0, -2.0);
    schedule.linear_ramp_to_value_at_time(10.0 / 4.0, 1.0);
    schedule.set_target_at_time(12.0 / 4.0, 0.0, 0.5);
    schedule.cancel_and_hold_at_time(15.0 / 4.0);
    schedule.exponential_ramp_to_value_at_time(19.0 / 4.0, 1.0);

    schedule.dispatch(100.0);

    let mut node = eq.finish(param);

    for _ in 0..20 {
        dbg!(pc.time);
        dbg!(node.proc(&pc));
        pc.time += 1.0 / pc.sample_rate as f64;
    }
}
