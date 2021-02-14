use std::{cell::RefCell, rc::Rc};

use super::{Node, ProcContext};

#[derive(Debug, Clone)]
pub enum EventBody {
    SetValueAtTime {value: f32},
    LinearRampToValueAtTime {value: f32},
    // ExponentialRampToValueAtTime {value: f32},
    SetTargetAtTime {target: f32, time_constant: f32},
}

#[derive(Debug, Clone)]
pub struct Event {
    time: f64,
    body: EventBody,
}

#[derive(Clone)]
pub struct Param {
    events: Rc<RefCell<Vec<Event>>>,
    current_value: f32,
}

#[derive(Clone)]
pub struct ParamController {
    events: Rc<RefCell<Vec<Event>>>,
}

impl Param {
    pub fn new() -> Self {
        Param {
            events: Rc::new(RefCell::new(Vec::new())),
            current_value: 0.0,
        }
    }

    pub fn controller(&self) -> ParamController {
        ParamController {
            events: self.events.clone(),
        }
    }
}

impl ParamController {
    pub fn push_event(&mut self, event: Event) {
        let mut events = self.events.borrow_mut();
        for (i, e) in events.iter().enumerate() {
            if event.time < e.time {
                events.insert(i, event);
                return;
            }
        }
        events.push(event);
    }

    pub fn set_value_at_time(&mut self, time: f64, value: f32) {
        self.push_event(Event {
            time,
            body: EventBody::SetValueAtTime {value},
        });
    }

    pub fn linera_ramp_to_value_at_time(&mut self, time: f64, value: f32) {
        self.push_event(Event {
            time,
            body: EventBody::LinearRampToValueAtTime {value},
        });
    }

    pub fn set_target_at_time(&mut self, time: f64, target: f32, time_constant: f32) {
        self.push_event(Event {
            time,
            body: EventBody::SetTargetAtTime {target, time_constant},
        });
    }

    pub fn cancel_scheduled_values(&mut self, time: f64) {
        let mut events = self.events.borrow_mut();
        if let Some(i) = events.iter().position(|e| time <= e.time) {
            events.truncate(i);
        }
    }

    // TODO: https://developer.mozilla.org/en-US/docs/Web/API/AudioParam/cancelAndHoldAtTime
}

impl Node<f32> for Param {
    fn proc(&mut self, ctx: &ProcContext) -> f32 {
        let mut events = self.events.borrow_mut();
        let mut canceled = false;
        while !events.is_empty() {
            let time = events[0].time;
            match events[0].body {
                EventBody::SetValueAtTime { value } => {
                    if time <= ctx.time {
                        self.current_value = value;
                        canceled = true;
                        events.remove(0);
                    } else {
                        break;
                    }
                }
                EventBody::LinearRampToValueAtTime { value } => {
                    if time <= ctx.time {
                        self.current_value = value;
                        canceled = true;
                        events.remove(0);
                    } else {
                        if !canceled {
                            self.current_value += (value - self.current_value) / ((time - ctx.time) as f32 * ctx.sample_rate as f32 + 1.0);
                        }
                        break;
                    }
                }
                EventBody::SetTargetAtTime { target, time_constant } => {
                    if time <= ctx.time {
                        if events.len() >= 2 && events[1].time <= ctx.time {
                            events.remove(0);
                            continue;
                        } else {
                            let factor = 1.0 / std::f32::consts::E.powf(1.0 / (ctx.sample_rate as f32 * time_constant));
                            self.current_value = (self.current_value - target) * factor + target;
                            break;
                        }
                    } else {
                        break;
                    }
                }
            }
        }
        self.current_value
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

    ctrl.push_event(Event {
        time: 2.0 / 4.0,
        body: EventBody::SetValueAtTime { value: 1.0 },
    });
    ctrl.push_event(Event {
        time: 4.0 / 4.0,
        body: EventBody::SetValueAtTime { value: 2.0 },
    });
    ctrl.push_event(Event {
        time: 6.0 / 4.0,
        body: EventBody::LinearRampToValueAtTime { value: -2.0 },
    });
    ctrl.push_event(Event {
        time: 10.0 / 4.0,
        body: EventBody::LinearRampToValueAtTime { value: 1.0 },
    });
    ctrl.push_event(Event {
        time: 12.0 / 4.0,
        body: EventBody::SetTargetAtTime {target: 0.0, time_constant: 0.5},
    });

    for _ in 0..15 {
        dbg!(pc.time);
        dbg!(param.proc(&pc));
        pc.time += 1.0 / pc.sample_rate as f64;
    }
}
