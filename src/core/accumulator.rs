use std::ops::{Add, Mul};

use crate::signal::{C1f64, Signal};

use super::{Node, ProcContext};

#[derive(Debug, Clone)]
pub struct Event<T>
where
    T: Signal + Mul<C1f64, Output = T> + Add<Output = T> + Default + Clone,
{
    time: f64,
    value: T,
}

pub struct Accumulator<T, A>
where
    T: Signal + Mul<C1f64, Output = T> + Add<Output = T> + Default + Clone,
    A: Node<T>,
{
    events: Vec<Event<T>>,
    node: A,
    value: T,
    upper: T,
}

impl<T, A> Accumulator<T, A>
where
    T: Signal + Mul<C1f64, Output = T> + Add<Output = T> + Default + Clone,
    A: Node<T>,
{
    pub fn new(node: A, upper: T) -> Self {
        Accumulator {
            events: vec![],
            node,
            value: Default::default(),
            upper,
        }
    }

    pub fn set_value_at_time(&mut self, time: f64, value: T) {
        let event = Event { time, value };
        for (i, e) in self.events.iter().enumerate() {
            if time < e.time {
                self.events.insert(i, event);
                return;
            }
        }
        self.events.push(event);
    }
}

impl<T, A> Node<T> for Accumulator<T, A>
where
    T: Signal<Float = f64> + Mul<C1f64, Output = T> + Add<Output = T> + Default + Clone,
    A: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let sample_rate = ctx.sample_rate as f64;
        let d = self.node.proc(ctx).map(|f| f / sample_rate);
        self.value = self.value.clone() + d;

        while !self.events.is_empty() {
            if ctx.time < self.events[0].time {
                break;
            }
            self.value = self.events[0].value.clone();
            self.events.remove(0);
        }

        self.value = self
            .value
            .map2_1(self.upper.clone(), |v, u| v.rem_euclid(u));
        self.value.clone()
    }

    fn lock(&mut self) {
        self.node.lock();
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

#[test]
fn test() {
    let mut accumulator = Accumulator::new(
        super::constant::Constant::new(C1f64::from(1.0)),
        C1f64::from(4.0),
    );
    let mut pc = ProcContext::new(4);

    accumulator.set_value_at_time(0.0, 1.0.into());
    accumulator.set_value_at_time(2.0, 0.5.into());
    accumulator.set_value_at_time(3.0, (-1.0).into());

    for _ in 0..20 {
        dbg!(pc.time);
        dbg!(accumulator.proc(&pc));
        pc.time += 1.0 / pc.sample_rate as f64;
    }
}
