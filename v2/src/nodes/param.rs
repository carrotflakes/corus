use crate::{EventQueue, PackedEvent, ProccessContext};

use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy)]
pub enum ParamState {
    Constant(f64),
    Linear(f64),
    Exponential(f64, f64),
    Target { target: f64, time_constant: f64 },
}

struct ParamInner {
    value: f64,
    sample_rate: f64,
    pre_add: f64,
    post_add: f64,
    mul: f64,
    state: ParamState,
}

impl ParamInner {
    fn process(&mut self, sample_rate: f64) -> f64 {
        if sample_rate != self.sample_rate {
            match self.state {
                ParamState::Constant(v) => {
                    self.pre_add = 0.0.into();
                    self.post_add = v;
                    self.mul = 0.0.into();
                }
                ParamState::Linear(v) => {
                    self.pre_add = 0.0.into();
                    self.post_add = v / sample_rate;
                    self.mul = 1.0.into();
                }
                ParamState::Exponential(v, vv) => {
                    self.pre_add = 0.0.into();
                    self.post_add = 0.0.into();
                    self.mul = v.powf(1.0 / (vv * sample_rate));
                }
                ParamState::Target {
                    target,
                    time_constant,
                } => {
                    self.pre_add = -target;
                    self.post_add = target;
                    self.mul = 1.0 / (1.0 / (time_constant * sample_rate)).exp();
                }
            }
            self.sample_rate = sample_rate;
        }
        let value = self.value;
        self.value = (self.pre_add + self.value) * self.mul + self.post_add;
        value
    }
}

pub struct Param {
    inner: Arc<ParamInner>,
    scheduler: Arc<Mutex<Scheduler>>,
}

impl Param {
    pub fn new(value: f64) -> Self {
        let inner = Arc::new(ParamInner {
            value,
            sample_rate: 0.0,
            pre_add: 0.0,
            post_add: 0.0,
            mul: 0.0,
            state: ParamState::Constant(value),
        });
        let scheduler = Scheduler {
            handles: Vec::new(),
            param_inner: inner.clone(),
            last_handle: (0.0, Handle::SetValue { value: 0.0 }),
            last_value: inner.value,
        };
        Self {
            inner,
            scheduler: Arc::new(Mutex::new(scheduler)),
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext) -> f64 {
        let inner = unsafe { std::mem::transmute::<_, &mut ParamInner>(Arc::as_ptr(&self.inner)) };
        inner.process(ctx.sample_rate())
    }

    pub fn scheduler(&self) -> Arc<Mutex<Scheduler>> {
        self.scheduler.clone()
    }
}

#[derive(Debug, Clone)]
enum Handle {
    SetValue { value: f64 },
    LinearRamp { value: f64 },
    ExponentialRamp { value: f64 },
    SetTarget { target: f64, time_constant: f64 },
}

pub struct Scheduler {
    handles: Vec<(f64, Handle)>,
    last_handle: (f64, Handle),
    last_value: f64,
    param_inner: Arc<ParamInner>,
}

impl Scheduler {
    fn push_handle(&mut self, time: f64, handle: Handle) {
        if let Some(index) = self.handles.iter().position(|(t, _)| *t > time) {
            self.handles.insert(index, (time, handle));
        } else {
            self.handles.push((time, handle));
        }
    }

    pub fn set_value_at_time(&mut self, time: f64, value: f64) {
        self.push_handle(time, Handle::SetValue { value });
    }

    pub fn linear_ramp_to_value_at_time(&mut self, time: f64, value: f64) {
        self.push_handle(time, Handle::LinearRamp { value });
    }

    pub fn exponential_ramp_to_value_at_time(&mut self, time: f64, value: f64) {
        assert!(1.0e-10 < value.abs() && value.is_finite());
        self.push_handle(time, Handle::ExponentialRamp { value });
    }

    pub fn set_target_at_time(&mut self, time: f64, target: f64, time_constant: f64) {
        self.push_handle(
            time,
            Handle::SetTarget {
                target,
                time_constant,
            },
        );
    }

    pub fn cancel_scheduled_values(&mut self, time: f64) {
        self.cancel_scheduled_values_(time);
    }

    fn cancel_scheduled_values_(&mut self, time: f64) -> Option<(f64, Handle)> {
        if let Some(index) = self.handles.iter().position(|(t, _)| *t > time) {
            let e = self.handles[index].clone();
            self.handles.truncate(index);
            Some(e)
        } else {
            None
        }
    }

    pub fn cancel_and_hold_at_time(&mut self, time: f64) {
        let value = self.compute_value(time);
        if let Some(e) = self.cancel_scheduled_values_(time) {
            match e.1 {
                Handle::SetValue { .. } | Handle::SetTarget { .. } => {
                    self.set_value_at_time(time, value)
                }
                Handle::LinearRamp { .. } => self.linear_ramp_to_value_at_time(time, value),
                Handle::ExponentialRamp { .. } => {
                    self.exponential_ramp_to_value_at_time(time, value)
                }
            }
        } else {
            self.set_value_at_time(time, value); // OK?
        }
    }

    pub fn compute_value(&self, time: f64) -> f64 {
        let mut before = Some(&self.last_handle);
        let mut after = None;
        for event in &self.handles {
            if time < event.0 {
                match event.1 {
                    Handle::SetValue { .. } | Handle::SetTarget { .. } => {}
                    Handle::LinearRamp { .. } | Handle::ExponentialRamp { .. } => {
                        after = Some(event);
                    }
                }
                break;
            }
            match event.1 {
                Handle::SetValue { .. } => {
                    before = Some(event);
                    after = None;
                }
                Handle::LinearRamp { .. } | Handle::ExponentialRamp { .. } => {
                    before = Some(event);
                    after = None;
                }
                Handle::SetTarget { .. } => {
                    after = Some(event);
                }
            }
        }
        if let Some(before) = before {
            let before_value = match before.1.clone() {
                Handle::SetValue { value }
                | Handle::LinearRamp { value }
                | Handle::ExponentialRamp { value } => value,
                Handle::SetTarget { .. } => {
                    unreachable!()
                }
            };
            if let Some(after) = after {
                match after.1.clone() {
                    Handle::SetValue { .. } => {
                        unreachable!()
                    }
                    Handle::LinearRamp { value } => {
                        let r = (time - before.0) / (after.0 - before.0);
                        before_value.linear_interpolate(value, r)
                    }
                    Handle::ExponentialRamp { value } => {
                        let r = (time - before.0) / (after.0 - before.0);
                        before_value.exponential_interpolate(value, r)
                    }
                    Handle::SetTarget {
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

    pub fn push_events(&mut self, event_queue: &mut EventQueue, time: f64, dtime: f64) {
        let end_time = time + dtime;
        while !self.handles.is_empty() {
            let first = &self.handles[0];
            if end_time < first.0 {
                break;
            }
            match first.1.clone() {
                Handle::SetValue { value } => {
                    event_queue.push(first.0, self.set_value_event(ParamState::Constant(value)));
                    self.last_value = value;
                }
                Handle::LinearRamp { value } => {
                    event_queue.push(
                        self.last_handle.0,
                        self.set_value_event(ParamState::Linear(
                            (value - self.last_value) / (first.0 - self.last_handle.0),
                        )),
                    );
                    event_queue.push(first.0, self.set_value_event(ParamState::Constant(value)));
                    self.last_value = value;
                }
                Handle::ExponentialRamp { value } => {
                    event_queue.push(
                        self.last_handle.0,
                        self.set_value_event(ParamState::Exponential(
                            value / self.last_value,
                            first.0 - self.last_handle.0,
                        )),
                    );
                    event_queue.push(first.0, self.set_value_event(ParamState::Constant(value)));
                    self.last_value = value;
                }
                Handle::SetTarget {
                    target,
                    time_constant,
                } => {
                    event_queue.push(
                        first.0,
                        self.set_value_event(ParamState::Target {
                            target,
                            time_constant,
                        }),
                    );
                }
            }
            self.last_handle = first.clone();
            self.handles.remove(0);
        }

        if let Some(first) = self.handles.first() {
            match first.1.clone() {
                Handle::LinearRamp { value } => {
                    // TODO: prevent push multiple times.
                    event_queue.push(
                        self.last_handle.0,
                        self.set_value_event(ParamState::Linear(
                            (value - self.last_value) / (first.0 - self.last_handle.0),
                        )),
                    );
                }
                Handle::ExponentialRamp { value } => {
                    // TODO: prevent push multiple times.
                    event_queue.push(
                        self.last_handle.0,
                        self.set_value_event(ParamState::Exponential(
                            value / self.last_value,
                            first.0 - self.last_handle.0,
                        )),
                    );
                }
                Handle::SetValue { .. } | Handle::SetTarget { .. } => {}
            }
        }
    }

    pub fn set_value_event(&self, state: ParamState) -> PackedEvent {
        let inner_arc = self.param_inner.clone();
        Box::new(move |_time: f64| {
            let inner =
                unsafe { std::mem::transmute::<_, &mut ParamInner>(Arc::as_ptr(&inner_arc)) };
            match state {
                ParamState::Constant(value) => inner.value = value,
                _ => {}
            }
            inner.state = state;
            inner.sample_rate = 0.0;
        })
    }
}

pub trait Float:
    'static
    + Clone
    + Copy
    + Default
    + From<f64>
    + Into<f64>
    + std::ops::Mul<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Add<Output = Self>
    + std::ops::Sub<Output = Self>
    + std::ops::Neg<Output = Self>
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
