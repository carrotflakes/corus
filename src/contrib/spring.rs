use std::marker::PhantomData;

use crate::{signal::C1f64, Event, Node, ProcContext};

pub struct Spring<A, B, C, D>
where
    A: Node<C1f64>,
    B: Node<C1f64>,
    C: Node<C1f64>,
    D: Node<C1f64>,
{
    frequency: A,
    decay: B,
    velocity_limit: C,
    target: D,
    displacement: f64,
    velocity: f64,
}

impl<A, B, C, D> Spring<A, B, C, D>
where
    A: Node<C1f64>,
    B: Node<C1f64>,
    C: Node<C1f64>,
    D: Node<C1f64>,
{
    pub fn new(frequency: A, decay: B, velocity_limit: C, target: D, displacement: f64) -> Self {
        Spring {
            frequency,
            decay,
            velocity_limit,
            target,
            displacement,
            velocity: 0.0,
        }
    }
}

impl<A, B, C, D> Node<C1f64> for Spring<A, B, C, D>
where
    A: Node<C1f64>,
    B: Node<C1f64>,
    C: Node<C1f64>,
    D: Node<C1f64>,
{
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let frequency = self.frequency.proc(ctx);
        let decay = self.decay.proc(ctx);
        let velocity_limit = self.velocity_limit.proc(ctx) / ctx.sample_rate as f64;
        let target = self.target.proc(ctx);

        let k = (frequency / ctx.sample_rate as f64 * std::f64::consts::PI * 2.0).powi(2);
        let d = decay.powf(1.0 / ctx.sample_rate as f64);

        self.displacement = self.displacement + self.velocity;
        self.displacement *= d;
        if 1.0 < self.displacement {
            self.displacement = 1.0;
            self.velocity = 0.0;
        } else if self.displacement < -1.0 {
            self.displacement = -1.0;
            self.velocity = 0.0;
        } else {
            self.velocity -= (self.displacement - target) * k;
            self.velocity = self.velocity.clamp(-velocity_limit, velocity_limit);
        }
        self.displacement
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.frequency.lock(ctx);
    }

    fn unlock(&mut self) {
        self.frequency.unlock();
    }
}

pub enum SpringEvent<A, B, C, D> {
    Reset(f64, f64),
    _T(
        PhantomData<A>,
        PhantomData<B>,
        PhantomData<C>,
        PhantomData<D>,
    ),
}

impl<A, B, C, D> Event for SpringEvent<A, B, C, D>
where
    A: 'static + Node<C1f64>,
    B: 'static + Node<C1f64>,
    C: 'static + Node<C1f64>,
    D: 'static + Node<C1f64>,
{
    type Target = Spring<A, B, C, D>;

    fn dispatch(&self, _time: f64, target: &mut Self::Target) {
        match self {
            SpringEvent::Reset(displacement, velocity) => {
                target.displacement = *displacement;
                target.velocity = *velocity;
            }
            SpringEvent::_T(_, _, _, _) => {}
        }
    }
}
