use std::marker::PhantomData;

use crate::{signal::C1f64, Event, Node, ProcContext};

pub struct Spring<A, B>
where
    A: Node<C1f64>,
    B: Node<C1f64>,
{
    frequency: A,
    decay: B,
    displacement: f64,
    velocity: f64,
}

impl<A, B> Spring<A, B>
where
    A: Node<C1f64>,
    B: Node<C1f64>,
{
    pub fn new(frequency: A, decay: B, displacement: f64) -> Self {
        Spring {
            frequency,
            decay,
            displacement,
            velocity: 0.0,
        }
    }
}

impl<A, B> Node<C1f64> for Spring<A, B>
where
    A: Node<C1f64>,
    B: Node<C1f64>,
{
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let frequency = self.frequency.proc(ctx);
        let decay = self.decay.proc(ctx);

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
            self.velocity -= self.displacement * k;
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

pub enum SpringEvent<A, B> {
    Reset(f64, f64),
    _T(PhantomData<A>, PhantomData<B>),
}

impl<A, B> Event for SpringEvent<A, B>
where
    A: 'static + Node<C1f64>,
    B: 'static + Node<C1f64>,
{
    type Target = Spring<A, B>;

    fn dispatch(&self, _time: f64, target: &mut Self::Target) {
        match self {
            SpringEvent::Reset(displacement, velocity) => {
                target.displacement = *displacement;
                target.velocity = *velocity;
            }
            SpringEvent::_T(_, _) => {}
        }
    }
}
