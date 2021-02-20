use crate::signal::C1f32;

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
    events: Vec<Event>,
}

impl Param {
    pub fn new() -> Self {
        Self::with_value(0.0)
    }

    pub fn with_value(value: f32) -> Self {
        Param {
            events: vec![Event {
                time: 0.0,
                body: EventBody::SetValueAtTime { value },
            }]
        }
    }

    pub fn push_event(&mut self, event: Event) {
        for (i, e) in self.events.iter().enumerate() {
            if event.time < e.time {
                self.events.insert(i, event);
                return;
            }
        }
        self.events.push(event);
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
        if let Some(i) = self.events.iter().position(|e| time <= e.time) {
            self.events.truncate(i);
        }
    }

    pub fn cancel_and_hold_at_time(&mut self, time: f64) {
        let value = self.compute_value(time);
        self.cancel_scheduled_values(time);
        self.set_value_at_time(time, value)
    }

    pub fn compute_value(&self, time: f64) -> f32 {
        let mut before = None;
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
}

impl Node<C1f32> for Param {
    fn proc(&mut self, ctx: &ProcContext) -> C1f32 {
        {
            while self.events.len() >= 2 {
                if ctx.time < self.events[1].time {
                    break;
                }
                match self.events[1].body {
                    EventBody::SetValueAtTime { .. }
                    | EventBody::LinearRampToValueAtTime { .. }
                    | EventBody::ExponentialRampToValueAtTime { .. } => {
                        self.events.remove(0);
                    }
                    EventBody::SetTargetAtTime { .. } => {
                        if let Some(e) = self.events.get(2) {
                            if ctx.time < e.time {
                                break;
                            }
                            match e.body {
                                EventBody::SetValueAtTime { .. }
                                | EventBody::LinearRampToValueAtTime { .. }
                                | EventBody::ExponentialRampToValueAtTime { .. } => {
                                    self.events.drain(0..2).count();
                                }
                                EventBody::SetTargetAtTime { .. } => {
                                    self.events.remove(1);
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        self.compute_value(ctx.time).into()
    }

    fn lock(&mut self) {
    }

    fn unlock(&mut self) {
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
