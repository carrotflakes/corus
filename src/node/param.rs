use std::{cell::RefCell, rc::Rc};

use super::{Node, ProcContext};

#[derive(Debug, Clone)]
pub enum EventBody {
    SetValueAtTime { value: f32 },
    LinearRampToValueAtTime { value: f32 },
    ExponentialRampToValueAtTime { value: f32 },
    SetTargetAtTime { target: f32, time_constant: f32 },
}

#[derive(Debug, Clone)]
pub struct Event {
    time: f64,
    body: EventBody,
}

pub struct Param {
    body: Rc<RefCell<ParamBody>>,
}

pub struct ParamBody {
    events: Vec<Event>,
}

impl Param {
    pub fn new() -> Self {
        Self::with_value(0.0)
    }

    pub fn with_value(value: f32) -> Self {
        Param {
            body: Rc::new(RefCell::new(ParamBody {
                events: vec![Event {
                    time: 0.0,
                    body: EventBody::SetValueAtTime { value },
                }],
            })),
        }
    }

    pub fn controller(&self) -> Param {
        Param {
            body: self.body.clone(),
        }
    }

    pub fn push_event(&mut self, event: Event) {
        let mut body = self.body.borrow_mut();
        for (i, e) in body.events.iter().enumerate() {
            if event.time < e.time {
                body.events.insert(i, event);
                return;
            }
        }
        body.events.push(event);
    }

    pub fn set_value_at_time(&mut self, time: f64, value: f32) {
        self.push_event(Event {
            time,
            body: EventBody::SetValueAtTime { value },
        });
    }

    pub fn linear_ramp_to_value_at_time(&mut self, time: f64, value: f32) {
        self.push_event(Event {
            time,
            body: EventBody::LinearRampToValueAtTime { value },
        });
    }

    pub fn exponential_ramp_to_value_at_time(&mut self, time: f64, value: f32) {
        self.push_event(Event {
            time,
            body: EventBody::ExponentialRampToValueAtTime { value },
        });
    }

    pub fn set_target_at_time(&mut self, time: f64, target: f32, time_constant: f32) {
        self.push_event(Event {
            time,
            body: EventBody::SetTargetAtTime {
                target,
                time_constant,
            },
        });
    }

    pub fn cancel_scheduled_values(&mut self, time: f64) {
        let mut body = self.body.borrow_mut();
        if let Some(i) = body.events.iter().position(|e| time <= e.time) {
            body.events.truncate(i);
        }
    }

    pub fn cancel_and_hold_at_time(&mut self, time: f64) {
        let value = self.compute_value(time);
        self.cancel_scheduled_values(time);
        self.set_value_at_time(time, value)
    }

    pub fn compute_value(&self, time: f64) -> f32 {
        let body = self.body.borrow();
        let mut before = None;
        let mut after = None;
        for event in &body.events {
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
                        let r = ((time - before.time) / (after.time - before.time)) as f32;
                        before_value * (1.0 - r) + value * r
                    }
                    EventBody::ExponentialRampToValueAtTime { value } => {
                        let r = ((time - before.time) / (after.time - before.time)) as f32;
                        (before_value.ln() * (1.0 - r) + value.ln() * r).exp()
                    }
                    EventBody::SetTargetAtTime {
                        target,
                        time_constant,
                    } => {
                        let t = (time - before.time) as f32;
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
    // TODO: https://developer.mozilla.org/en-US/docs/Web/API/AudioParam/cancelAndHoldAtTime
}

impl Node<f32> for Param {
    fn proc(&mut self, ctx: &ProcContext) -> f32 {
        {
            let mut body = self.body.borrow_mut();

            while body.events.len() >= 2 {
                if ctx.time < body.events[1].time {
                    break;
                }
                match body.events[1].body {
                    EventBody::SetValueAtTime { .. }
                    | EventBody::LinearRampToValueAtTime { .. }
                    | EventBody::ExponentialRampToValueAtTime { .. } => {
                        body.events.remove(0);
                    }
                    EventBody::SetTargetAtTime { .. } => {
                        if let Some(e) = body.events.get(2) {
                            if ctx.time < e.time {
                                break;
                            }
                            match e.body {
                                EventBody::SetValueAtTime { .. }
                                | EventBody::LinearRampToValueAtTime { .. }
                                | EventBody::ExponentialRampToValueAtTime { .. } => {
                                    body.events.drain(0..2).count();
                                }
                                EventBody::SetTargetAtTime { .. } => {
                                    body.events.remove(1);
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        self.compute_value(ctx.time)
    }
}

impl AsMut<Param> for Param {
    fn as_mut(&mut self) -> &mut Param {
        self
    }
}

#[test]
fn test() {
    let mut param = Param::new();
    let mut ctrl = param.controller();
    let mut pc = ProcContext::new(4);

    ctrl.set_value_at_time(2.0 / 4.0, 1.0);
    ctrl.set_value_at_time(4.0 / 4.0, 2.0);
    ctrl.linear_ramp_to_value_at_time(6.0 / 4.0, -2.0);
    ctrl.linear_ramp_to_value_at_time(10.0 / 4.0, 1.0);
    ctrl.set_target_at_time(12.0 / 4.0, 0.0, 0.5);
    ctrl.cancel_and_hold_at_time(15.0 / 4.0);
    ctrl.exponential_ramp_to_value_at_time(19.0 / 4.0, 1.0);

    for _ in 0..20 {
        dbg!(pc.time);
        dbg!(param.proc(&pc));
        pc.time += 1.0 / pc.sample_rate as f64;
    }
}
