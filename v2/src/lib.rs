pub mod nodes;
pub mod unsafe_wrapper;

#[path = "./../../src/ring_buffer.rs"]
pub mod ring_buffer;

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct ProccessContext {
    sample_rate: f64,
    dtime: f64,
    current_time: f64,
}

impl ProccessContext {
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

pub type PackedEvent = Box<dyn FnOnce(f64)>;

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
    type Output;

    fn process(&mut self, ctx: &ProccessContext) -> Self::Output;
}
