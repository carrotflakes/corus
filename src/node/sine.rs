use crate::signal::C1f32;

use super::{Node, ProcContext};

pub struct Sine<A, DA>
where
    A: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
{
    frequency: DA,
    phase: C1f32,
    _a: std::marker::PhantomData<A>,
}

impl<A, DA> Sine<A, DA>
where
    A: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(frequency: DA) -> Self {
        Sine {
            frequency,
            phase: 0.0.into(),
            _a: Default::default(),
        }
    }
}

impl<A, DA> Node<C1f32> for Sine<A, DA>
where
    A: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
{
    fn proc(&mut self, ctx: &ProcContext) -> C1f32 {
        let f = self.frequency.as_mut().proc(ctx);
        let p = self.phase;
        self.phase = (self.phase.0[0] + f.0[0] / ctx.sample_rate as f32).fract().into();
        (p.0[0] * std::f32::consts::PI * 2.0).sin().into()
    }

    fn lock(&mut self) {
        self.frequency.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.frequency.as_mut().unlock();
    }
}

impl<A, DA> AsMut<Self> for Sine<A, DA>
where
    A: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
