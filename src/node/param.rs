use std::{collections::VecDeque, marker::PhantomData};

use crate::signal::Mono;

use super::{Node, ProcContext};

#[derive(Debug, Clone)]
pub enum EventBody<F: 'static + Clone + Default> {
    SetValueAtTime { value: F },
    LinearRampToValueAtTime { value: F },
    ExponentialRampToValueAtTime { value: F },
    SetTargetAtTime { target: F, time_constant: F },
}

#[derive(Debug, Clone)]
pub struct Event<F: 'static + Clone + Default> {
    time: f64,
    body: EventBody<F>,
}

pub struct Param<F: 'static + Clone + Default, T: Mono<F>> {
    first_event: Event<F>,
    events: VecDeque<Event<F>>,
    _t: PhantomData<T>,
}

impl<T: Mono<f64>> Param<f64, T> {
    pub fn new() -> Self {
        Self::with_value(Default::default())
    }

    pub fn with_value(value: f64) -> Self {
        Param {
            first_event: Event {
                time: 0.0,
                body: EventBody::SetValueAtTime { value },
            },
            events: vec![].into(),
            _t: Default::default(),
        }
    }

    pub fn push_event(&mut self, event: Event<f64>) {
        for (i, e) in self.events.iter().enumerate() {
            if event.time < e.time {
                self.events.insert(i, event);
                return;
            }
        }
        self.events.push_back(event);
    }

    pub fn set_value_at_time(&mut self, time: f64, value: f64) {
        self.push_event(Event {
            time,
            body: EventBody::SetValueAtTime { value },
        });
    }

    pub fn linear_ramp_to_value_at_time(&mut self, time: f64, value: f64) {
        self.push_event(Event {
            time,
            body: EventBody::LinearRampToValueAtTime { value },
        });
    }

    pub fn exponential_ramp_to_value_at_time(&mut self, time: f64, value: f64) {
        self.push_event(Event {
            time,
            body: EventBody::ExponentialRampToValueAtTime { value },
        });
    }

    pub fn set_target_at_time(&mut self, time: f64, target: f64, time_constant: f64) {
        self.push_event(Event {
            time,
            body: EventBody::SetTargetAtTime {
                target,
                time_constant,
            },
        });
    }

    fn cancel_scheduled_values_(&mut self, time: f64) -> Option<Event<f64>> {
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
        }
    }

    pub fn compute_value(&self, time: f64) -> f64 {
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
            let before_value = match before.body {
                EventBody::SetValueAtTime { value }
                | EventBody::LinearRampToValueAtTime { value }
                | EventBody::ExponentialRampToValueAtTime { value } => value,
                EventBody::SetTargetAtTime { .. } => {
                    unreachable!()
                }
            };
            if let Some(after) = after {
                match after.body {
                    EventBody::SetValueAtTime { .. } => {
                        unreachable!()
                    }
                    EventBody::LinearRampToValueAtTime { value } => {
                        let r = ((time - before.time) / (after.time - before.time)) as f64;
                        before_value * (1.0 - r) + value * r
                    }
                    EventBody::ExponentialRampToValueAtTime { value } => {
                        let r = ((time - before.time) / (after.time - before.time)) as f64;
                        (before_value.ln() * (1.0 - r) + value.ln() * r).exp()
                    }
                    EventBody::SetTargetAtTime {
                        target,
                        time_constant,
                    } => {
                        let t = (time - before.time) as f64;
                        let r = (-t / time_constant).exp();
                        before_value * r + target * (1.0 - r)
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

impl<T: Mono<f64>> Node<T> for Param<f64, T> {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        {
            while !self.events.is_empty() {
                if ctx.time < self.events[0].time {
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
                            if ctx.time < e.time {
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

        T::from_m(self.compute_value(ctx.time))
    }

    fn lock(&mut self) {}

    fn unlock(&mut self) {}
}

impl<F: 'static + Clone + Default, T: Mono<F>> AsMut<Param<F, T>> for Param<F, T> {
    fn as_mut(&mut self) -> &mut Param<F, T> {
        self
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
        dbg!(pc.time);
        dbg!(param.proc(&pc));
        pc.time += 1.0 / pc.sample_rate as f64;
    }
}
