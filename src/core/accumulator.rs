use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

use crate::{
    signal::{C1f64, Signal},
    Event,
};

use super::{Node, ProcContext};

pub struct Accumulator<T, A>
where
    T: Signal + Mul<C1f64, Output = T> + Add<Output = T> + Default + Clone,
    A: Node<T>,
{
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
            node,
            value: Default::default(),
            upper,
        }
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

pub struct SetValueAtTime<T, A>
where
    T: Signal + Mul<C1f64, Output = T> + Add<Output = T> + Default + Clone,
    A: Node<T> + 'static,
{
    value: T,
    _t: PhantomData<A>,
}

impl<T, A> SetValueAtTime<T, A>
where
    T: Signal + Mul<C1f64, Output = T> + Add<Output = T> + Default + Clone,
    A: Node<T> + 'static,
{
    pub fn new(value: T) -> Self {
        Self {
            value,
            _t: Default::default(),
        }
    }
}

impl<T, A> Event for SetValueAtTime<T, A>
where
    T: Signal + Mul<C1f64, Output = T> + Add<Output = T> + Default + Clone,
    A: Node<T> + 'static,
{
    type Target = Accumulator<T, A>;

    fn dispatch(&self, time: f64, target: &mut Self::Target) {
        target.value = self.value.clone();
    }
}

#[test]
fn test() {
    use crate::EventControlInplace;
    let mut accumulator = EventControlInplace::new(Accumulator::new(
        super::constant::Constant::new(C1f64::from(1.0)),
        C1f64::from(4.0),
    ));
    let mut pc = ProcContext::new(4);

    accumulator.push_event(0.0, SetValueAtTime::new(1.0));
    accumulator.push_event(2.0, SetValueAtTime::new(0.5));
    accumulator.push_event(3.0, SetValueAtTime::new(-1.0));

    for _ in 0..20 {
        dbg!(pc.time);
        dbg!(accumulator.proc(&pc));
        pc.time += 1.0 / pc.sample_rate as f64;
    }
}
