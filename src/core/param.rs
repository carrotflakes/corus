use std::{collections::VecDeque, marker::PhantomData};

use crate::signal::Mono;

use super::{Node, ProcContext};

#[derive(Debug, Clone)]
pub enum EventBody<F: Float> {
    SetValueAtTime { value: F },
    LinearRampToValueAtTime { value: F },
    ExponentialRampToValueAtTime { value: F },
    SetTargetAtTime { target: F, time_constant: f64 },
}

#[derive(Debug, Clone)]
pub struct Event<F: Float> {
    time: f64,
    body: EventBody<F>,
}

pub struct Param<F: Float, T: Mono<F>> {
    first_event: Event<F>,
    events: VecDeque<Event<F>>,
    _t: PhantomData<T>,
}

impl<F: Float, T: Mono<F>> Param<F, T> {
    pub fn new() -> Self {
        Self::with_value(Default::default())
    }

    pub fn with_value(value: F) -> Self {
        Param {
            first_event: Event {
                time: 0.0,
                body: EventBody::SetValueAtTime { value },
            },
            events: vec![].into(),
            _t: Default::default(),
        }
    }

    pub fn push_event(&mut self, event: Event<F>) {
        for (i, e) in self.events.iter().enumerate() {
            if event.time < e.time {
                self.events.insert(i, event);
                return;
            }
        }
        self.events.push_back(event);
    }

    pub fn set_value_at_time(&mut self, time: f64, value: F) {
        self.push_event(Event {
            time,
            body: EventBody::SetValueAtTime { value },
        });
    }

    pub fn linear_ramp_to_value_at_time(&mut self, time: f64, value: F) {
        self.push_event(Event {
            time,
            body: EventBody::LinearRampToValueAtTime { value },
        });
    }

    pub fn exponential_ramp_to_value_at_time(&mut self, time: f64, value: F) {
        self.push_event(Event {
            time,
            body: EventBody::ExponentialRampToValueAtTime { value },
        });
    }

    pub fn set_target_at_time(&mut self, time: f64, target: F, time_constant: f64) {
        self.push_event(Event {
            time,
            body: EventBody::SetTargetAtTime {
                target,
                time_constant,
            },
        });
    }

    fn cancel_scheduled_values_(&mut self, time: f64) -> Option<Event<F>> {
        if let Some(i) = self.events.iter().position(|e| time <= e.time) {
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
            match e.body {
                EventBody::SetValueAtTime { .. } | EventBody::SetTargetAtTime { .. } => {
                    self.set_value_at_time(time, value)
                }
                EventBody::LinearRampToValueAtTime { .. } => {
                    self.linear_ramp_to_value_at_time(time, value)
                }
                EventBody::ExponentialRampToValueAtTime { .. } => {
                    self.exponential_ramp_to_value_at_time(time, value)
                }
            }
        } else {
            self.set_value_at_time(time, value); // OK?
        }
    }

    #[inline]
    pub fn compute_value(&self, time: f64) -> F {
        let mut before = Some(&self.first_event);
        let mut after = None;
        for event in &self.events {
            if time < event.time {
                match event.body {
                    EventBody::SetValueAtTime { .. } => {}
                    EventBody::LinearRampToValueAtTime { .. }
                    | EventBody::ExponentialRampToValueAtTime { .. } => {
                        after = Some(event);
                    }
                    EventBody::SetTargetAtTime { .. } => {}
                }
                break;
            }
            match event.body {
                EventBody::SetValueAtTime { .. } => {
                    before = Some(event);
                    after = None;
                }
                EventBody::LinearRampToValueAtTime { .. }
                | EventBody::ExponentialRampToValueAtTime { .. } => {
                    before = Some(event);
                    after = None;
                }
                EventBody::SetTargetAtTime { .. } => {
                    after = Some(event);
                }
            }
        }
        if let Some(before) = before {
            let before_value = match before.body.clone() {
                EventBody::SetValueAtTime { value }
                | EventBody::LinearRampToValueAtTime { value }
                | EventBody::ExponentialRampToValueAtTime { value } => value,
                EventBody::SetTargetAtTime { .. } => {
                    unreachable!()
                }
            };
            if let Some(after) = after {
                match after.body.clone() {
                    EventBody::SetValueAtTime { .. } => {
                        unreachable!()
                    }
                    EventBody::LinearRampToValueAtTime { value } => {
                        let r = (time - before.time) / (after.time - before.time);
                        before_value.linear_interpolate(value, r)
                    }
                    EventBody::ExponentialRampToValueAtTime { value } => {
                        let r = (time - before.time) / (after.time - before.time);
                        before_value.exponential_interpolate(value, r)
                    }
                    EventBody::SetTargetAtTime {
                        target,
                        time_constant,
                    } => {
                        let t = (time - after.time) as f64;
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
}

impl<T: Mono<f64>> Node for Param<f64, T> {
    type Output = T;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        {
            while !self.events.is_empty() {
                if ctx.current_time < self.events[0].time {
                    break;
                }
                match self.events[0].body {
                    EventBody::SetValueAtTime { .. }
                    | EventBody::LinearRampToValueAtTime { .. }
                    | EventBody::ExponentialRampToValueAtTime { .. } => {
                        self.first_event = self.events.pop_front().unwrap();
                    }
                    EventBody::SetTargetAtTime { .. } => {
                        if let Some(e) = self.events.get(1) {
                            if ctx.current_time < e.time {
                                break;
                            }
                            match e.body {
                                EventBody::SetValueAtTime { .. }
                                | EventBody::LinearRampToValueAtTime { .. }
                                | EventBody::ExponentialRampToValueAtTime { .. } => {
                                    self.events.pop_front();
                                    self.first_event = self.events.pop_front().unwrap();
                                }
                                EventBody::SetTargetAtTime { .. } => {
                                    self.events.pop_front();
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        T::from_m(self.compute_value(ctx.current_time))
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
}

pub trait Float: 'static + Clone + Default + From<f64> + Into<f64> {
    fn linear_interpolate(&self, other: Self, r: f64) -> Self;
    fn exponential_interpolate(&self, other: Self, r: f64) -> Self;
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
}

#[test]
fn test() {
    let mut param = Param::<f64, f64>::new();
    let mut pc = ProcContext::new(4);

    param.set_value_at_time(2.0 / 4.0, 1.0);
    param.set_value_at_time(4.0 / 4.0, 2.0);
    param.linear_ramp_to_value_at_time(6.0 / 4.0, -2.0);
    param.linear_ramp_to_value_at_time(10.0 / 4.0, 1.0);
    param.set_target_at_time(12.0 / 4.0, 0.0, 0.5);
    param.cancel_and_hold_at_time(15.0 / 4.0);
    param.exponential_ramp_to_value_at_time(19.0 / 4.0, 1.0);

    for _ in 0..20 {
        dbg!(pc.current_time);
        dbg!(param.proc(&pc));
        pc.current_time += 1.0 / pc.sample_rate as f64;
    }
}
