use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

use crate::{
    signal::{C1f64, Signal},
    EventListener,
};

use super::{Node, ProcContext};

pub struct Accumulator<A>
where
    A: Node,
    A::Output: Signal + Mul<C1f64, Output = A::Output> + Add<Output = A::Output> + Default + Clone,
{
    node: A,
    value: A::Output,
    upper: A::Output,
}

impl<A> Accumulator<A>
where
A: Node,
A::Output: Signal + Mul<C1f64, Output = A::Output> + Add<Output = A::Output> + Default + Clone,
{
    pub fn new(node: A, upper: A::Output) -> Self {
        Accumulator {
            node,
            value: Default::default(),
            upper,
        }
    }
}

impl<A> Node for Accumulator<A>
where
A: Node,
A::Output: Signal<Float = f64> + Mul<C1f64, Output = A::Output> + Add<Output = A::Output> + Default + Clone,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let sample_rate = ctx.sample_rate as f64;
        let d = self.node.proc(ctx).map(|f| f / sample_rate);
        self.value = self.value.clone() + d;

        self.value = self
            .value
            .map2_1(self.upper.clone(), |v, u| v.rem_euclid(u));
        self.value.clone()
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

pub struct SetValueAtTime<A>
where
A: Node,
A::Output: Signal + Mul<C1f64, Output = A::Output> + Add<Output = A::Output> + Default + Clone,
{
    value: A::Output,
    _t: PhantomData<A>,
}

impl<A> SetValueAtTime<A>
where
A: Node,
A::Output: Signal + Mul<C1f64, Output = A::Output> + Add<Output = A::Output> + Default + Clone,
{
    pub fn new(value: A::Output) -> Self {
        Self {
            value,
            _t: Default::default(),
        }
    }
}

impl<A> EventListener<SetValueAtTime<A>> for Accumulator<A>
where
A: 'static + Node,
A::Output: Signal + Mul<C1f64, Output = A::Output> + Add<Output = A::Output> + Default + Clone,
{
    #[inline]
    fn apply_event(&mut self, _time: f64, event: &SetValueAtTime<A>) {
        self.value = event.value.clone();
    }
}

#[test]
fn test() {
    use crate::{EventControlInplace, EventPusher};
    let mut accumulator = EventControlInplace::new(Accumulator::new(
        super::constant::Constant::new(C1f64::from(1.0)),
        C1f64::from(4.0),
    ));
    let mut pc = ProcContext::new(4);

    accumulator.push_event(0.0, SetValueAtTime::new(1.0));
    accumulator.push_event(2.0, SetValueAtTime::new(0.5));
    accumulator.push_event(3.0, SetValueAtTime::new(-1.0));

    for _ in 0..20 {
        dbg!(pc.current_time);
        dbg!(accumulator.proc(&pc));
        pc.current_time += 1.0 / pc.sample_rate as f64;
    }
}
