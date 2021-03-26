use std::marker::PhantomData;

use crate::EventListener;

use super::{Node, ProcContext};

pub struct Accumulator<A>
where
    A: Node<Output = f64>,
{
    node: A,
    value: A::Output,
    upper: A::Output,
}

impl<A> Accumulator<A>
where
    A: Node<Output = f64>,
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
    A: Node<Output = f64>,
{
    type Output = A::Output;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> Self::Output {
        let d = self.node.proc(ctx) / ctx.sample_rate as f64;
        self.value = (self.value + d).rem_euclid(self.upper);
        self.value
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
    A: Node<Output = f64>,
{
    value: A::Output,
    _t: PhantomData<A>,
}

impl<A> SetValueAtTime<A>
where
    A: Node<Output = f64>,
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
    A: Node<Output = f64> + 'static,
{
    #[inline]
    fn apply_event(&mut self, _time: f64, event: &SetValueAtTime<A>) {
        self.value = event.value.clone();
    }
}

#[test]
fn test() {
    use crate::{EventControlInplace, EventPusher};
    let mut accumulator =
        EventControlInplace::new(Accumulator::new(super::constant::Constant::new(1.0), 4.0));
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
