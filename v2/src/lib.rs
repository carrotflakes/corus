pub mod event_queue;
pub mod nodes;
pub mod ring_buffer;
pub mod signal;
pub mod unsafe_wrapper;

use std::collections::VecDeque;

use num_traits::{FromPrimitive, ToPrimitive};
use signal::Signal;

#[derive(Debug, Clone)]
pub struct ProcessContext {
    sample_rate: f64,
    dtime: f64,
    current_time: f64,
}

impl ProcessContext {
    pub fn new(sample_rate: f64) -> Self {
        Self {
            sample_rate,
            dtime: 1.0 / sample_rate,
            current_time: 0.0,
        }
    }

    pub fn sample_rate(&self) -> f64 {
        self.sample_rate
    }

    pub fn dtime(&self) -> f64 {
        self.dtime
    }

    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    pub fn next(&mut self) {
        self.current_time += self.dtime;
    }
}

pub type PackedEvent = Box<dyn FnOnce(f64) + Send + Sync>;

#[deprecated]
pub struct EventQueue {
    queue: VecDeque<(f64, PackedEvent)>,
}

impl EventQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, time: f64, event: PackedEvent) {
        if let Some(index) = self.queue.iter().position(|(t, _)| *t > time) {
            self.queue.insert(index, (time, event));
        } else {
            self.queue.push_back((time, event));
        }
    }

    pub fn dispatch(&mut self, current_time: f64) {
        let Some(x) = self.queue.front_mut() else {
            return;
        };
        if current_time < x.0 {
            return;
        }
        let x = self.queue.pop_front().unwrap();
        x.1(x.0);
        self.dispatch(current_time);
    }
}

pub trait Producer {
    type Output: Signal;

    fn process(&mut self, ctx: &ProcessContext) -> Self::Output;
}

#[inline]
pub fn interpolate_get<T: Signal>(x: T::Float, getter: impl Fn(usize) -> T) -> T
where
    T::Float: FromPrimitive + ToPrimitive,
{
    let x = x.to_f64().unwrap();
    let x0 = x.floor() as usize;
    let x1 = x0 + 1;
    let y0 = getter(x0);
    let y1 = getter(x1);
    let t = x - x0 as f64;
    y0 + (y1 - y0) * T::Float::from_f64(t).unwrap()
}

#[test]
fn test_interpolate_get() {
    let getter = |x| x as f64;
    assert_eq!(interpolate_get(0.0, getter), 0.0);
    assert_eq!(interpolate_get(0.5, getter), 0.5);
    assert_eq!(interpolate_get(0.9, getter), 0.9);
    assert_eq!(interpolate_get(1.0, getter), 1.0);
    assert_eq!(interpolate_get(1.5, getter), 1.5);
    assert_eq!(interpolate_get(1.9, getter), 1.9);
    assert_eq!(interpolate_get(2.0, getter), 2.0);
}
