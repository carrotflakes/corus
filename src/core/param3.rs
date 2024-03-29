use std::{
    collections::VecDeque,
    ops::{Add, Div, Mul, Neg, Sub},
    sync::{Arc, Mutex},
};

use crate::{EventControllable, EventListener, EventQueue};

use super::{Node, ProcContext};

#[derive(Clone, Copy)]
pub enum ParamState<F: Float> {
    Constant(F),
    Linear(F),
    Exponential(F, f64),
    Target { target: F, time_constant: f64 },
}

pub struct Param<F: Float> {
    value: F,
    sample_rate: u64,
    pre_add: F,
    post_add: F,
    mul: F,
    state: ParamState<F>,
}

impl<F: Float> Param<F> {
    pub fn new() -> Self {
        Self::from_value(Default::default())
    }

    pub fn from_value(value: F) -> Self {
        Param {
            value: value.clone(),
            sample_rate: 0,
            pre_add: 0.0.into(),
            post_add: 0.0.into(),
            mul: 1.0.into(),
            state: ParamState::Constant(value),
        }
    }
}

impl<F: Float> Node for Param<F> {
    type Output = F;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> F {
        if ctx.sample_rate != self.sample_rate {
            match self.state {
                ParamState::Constant(v) => {
                    self.pre_add = 0.0.into();
                    self.post_add = v;
                    self.mul = 0.0.into();
                }
                ParamState::Linear(v) => {
                    self.pre_add = 0.0.into();
                    self.post_add = v / F::from(ctx.sample_rate as f64);
                    self.mul = 1.0.into();
                }
                ParamState::Exponential(v, vv) => {
                    self.pre_add = 0.0.into();
                    self.post_add = 0.0.into();
                    self.mul = v.powf(F::from(1.0 / (vv * ctx.sample_rate as f64)));
                }
                ParamState::Target {
                    target,
                    time_constant,
                } => {
                    self.pre_add = -target;
                    self.post_add = target;
                    self.mul =
                        F::from(1.0 / (1.0 / (time_constant * ctx.sample_rate as f64)).exp());
                }
            }
            self.sample_rate = ctx.sample_rate;
        }
        let value = self.value.clone();
        self.value = (self.pre_add + self.value) * self.mul + self.post_add;
        value
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
}

pub trait Float:
    'static
    + Clone
    + Copy
    + Default
    + From<f64>
    + Into<f64>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Neg<Output = Self>
    + Send
    + Sync
    + std::fmt::Debug
    + PartialOrd
{
    fn linear_interpolate(&self, other: Self, r: f64) -> Self;
    fn exponential_interpolate(&self, other: Self, r: f64) -> Self;
    fn powf(&self, other: Self) -> Self;
    fn is_finite(self) -> bool;
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

    #[inline]
    fn is_finite(self) -> bool {
        f64::is_finite(self)
    }
}

impl<F: Float> EventListener<ParamState<F>> for Param<F> {
    #[inline]
    fn apply_event(&mut self, _time: f64, event: &ParamState<F>) {
        match event {
            ParamState::Constant(value) => {
                self.value = value.clone();
            }
            _ => {}
        }
        self.sample_rate = 0;
        self.state = event.clone();
    }
}

#[derive(Debug, Clone)]
enum ParamEvent<F: Float> {
    SetValueAtTime { value: F },
    LinearRampToValueAtTime { value: F },
    ExponentialRampToValueAtTime { value: F },
    SetTargetAtTime { target: F, time_constant: f64 },
}

pub struct ParamEventSchedule<F: Float> {
    events: VecDeque<(f64, ParamEvent<F>)>,
    last_event: (f64, ParamEvent<F>),
    last_value: F,
}

impl<F: Float + Send + Sync> ParamEventSchedule<F> {
    pub fn new() -> Self {
        Self {
            events: VecDeque::new(),
            last_event: (
                0.0,
                ParamEvent::SetValueAtTime {
                    value: F::from(0.0),
                },
            ),
            last_value: F::from(0.0),
        }
    }

    fn push_event(&mut self, time: f64, event: ParamEvent<F>) {
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
        assert!(F::from(0.0) < value && value.is_finite());
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

    pub fn send(
        &mut self,
        time: f64,
        event_queue: &EventQueue,
        param: &EventControllable<Param<F>>,
    ) {
        while !self.events.is_empty() {
            let first = &self.events[0];
            if time < first.0 {
                break;
            }
            match first.1.clone() {
                ParamEvent::SetValueAtTime { value } => {
                    event_queue.push_event(
                        first.0,
                        ParamState::Constant(value.clone()),
                        param.inner(),
                    );
                    self.last_value = value;
                }
                ParamEvent::LinearRampToValueAtTime { value } => {
                    event_queue.push_event(
                        self.last_event.0,
                        ParamState::Linear(
                            (value.clone() - self.last_value)
                                / F::from(first.0 - self.last_event.0),
                        ),
                        param.inner(),
                    );
                    event_queue.push_event(
                        first.0,
                        ParamState::Constant(value.clone()),
                        param.inner(),
                    );
                    self.last_value = value;
                }
                ParamEvent::ExponentialRampToValueAtTime { value } => {
                    event_queue.push_event(
                        self.last_event.0,
                        ParamState::Exponential(
                            value.clone() / self.last_value,
                            first.0 - self.last_event.0,
                        ),
                        param.inner(),
                    );
                    event_queue.push_event(
                        first.0,
                        ParamState::Constant(value.clone()),
                        param.inner(),
                    );
                    self.last_value = value;
                }
                ParamEvent::SetTargetAtTime {
                    target,
                    time_constant,
                } => {
                    event_queue.push_event(
                        first.0,
                        ParamState::Target {
                            target,
                            time_constant,
                        },
                        param.inner(),
                    );
                }
            }
            self.last_event = first.clone();
            self.events.pop_front();
        }

        if let Some(first) = self.events.front() {
            match first.1.clone() {
                ParamEvent::SetValueAtTime { .. } => {}
                ParamEvent::LinearRampToValueAtTime { value } => {
                    // TODO: prevent push multiple times.
                    event_queue.push_event(
                        self.last_event.0,
                        ParamState::Linear(
                            (value.clone() - self.last_value)
                                / F::from(first.0 - self.last_event.0),
                        ),
                        param.inner(),
                    );
                }
                ParamEvent::ExponentialRampToValueAtTime { value } => {
                    // TODO: prevent push multiple times.
                    event_queue.push_event(
                        self.last_event.0,
                        ParamState::Exponential(
                            value.clone() / self.last_value,
                            first.0 - self.last_event.0,
                        ),
                        param.inner(),
                    );
                }
                ParamEvent::SetTargetAtTime { .. } => {}
            }
        }
    }
}

pub struct ParamEventScheduleNode<F: Float> {
    param: EventControllable<Param<F>>,
    schedule: Arc<Mutex<ParamEventSchedule<F>>>,
}

impl<F: Float> ParamEventScheduleNode<F> {
    pub fn new() -> Self {
        Self::from_value(F::default())
    }

    pub fn from_value(value: F) -> Self {
        let param = EventControllable::new(Param::from_value(value));
        ParamEventScheduleNode {
            schedule: Arc::new(Mutex::new(ParamEventSchedule::new())),
            param,
        }
    }

    pub fn get_scheduler(&self) -> Arc<Mutex<ParamEventSchedule<F>>> {
        self.schedule.clone()
    }
}

impl<F: Float> Node for ParamEventScheduleNode<F> {
    type Output = F;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> F {
        self.param.proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.schedule.lock().unwrap().send(
            ctx.current_time + ctx.rest_proc_samples as f64 / ctx.sample_rate as f64,
            &ctx.event_queue,
            &self.param,
        );
        self.param.lock(ctx);
    }

    fn unlock(&mut self) {
        self.param.unlock();
    }
}

#[test]
fn test() {
    let mut eq = crate::EventQueue::new();
    let param = EventControllable::new(Param::<f64>::new());
    let mut schedule = ParamEventSchedule::new();
    let mut pc = ProcContext::new(4);

    schedule.set_value_at_time(2.0 / 4.0, 1.0);
    schedule.set_value_at_time(4.0 / 4.0, 2.0);
    schedule.linear_ramp_to_value_at_time(6.0 / 4.0, -2.0);
    schedule.linear_ramp_to_value_at_time(10.0 / 4.0, 1.0);
    schedule.set_target_at_time(12.0 / 4.0, 0.0, 0.5);
    schedule.cancel_and_hold_at_time(15.0 / 4.0);
    schedule.exponential_ramp_to_value_at_time(19.0 / 4.0, 1.0);

    schedule.send(100.0, &mut eq, &param);

    let mut node = param;
    pc.event_queue = eq;

    let mut lock = pc.lock(&mut node, crate::time::Second(10.0));
    for _ in 0..20 {
        dbg!(lock.next());
    }
}

#[test]
fn test2() {
    let param = ParamEventScheduleNode::new();
    let mut pc = ProcContext::new(4);
    {
        let schedule = param.get_scheduler();
        let mut schedule = schedule.lock().unwrap();
        schedule.set_value_at_time(2.0 / 4.0, 1.0);
        schedule.set_value_at_time(4.0 / 4.0, 2.0);
        schedule.linear_ramp_to_value_at_time(6.0 / 4.0, -2.0);
        schedule.linear_ramp_to_value_at_time(10.0 / 4.0, 1.0);
        schedule.set_target_at_time(12.0 / 4.0, 0.0, 0.5);
        schedule.cancel_and_hold_at_time(15.0 / 4.0);
        schedule.exponential_ramp_to_value_at_time(19.0 / 4.0, 1.0);
    }
    let mut node = param;

    let mut lock = pc.lock(&mut node, crate::time::Second(10.0));
    for _ in 0..20 {
        dbg!(lock.next());
    }
}
