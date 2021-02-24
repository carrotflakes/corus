use crate::signal::{C1f64, Mono};

use super::{Node, ProcContext};

pub struct Sine<A, DA>
where
    A: Node<C1f64> + ?Sized,
    DA: AsMut<A>,
{
    frequency: DA,
    phase: C1f64,
    _a: std::marker::PhantomData<A>,
}

impl<A, DA> Sine<A, DA>
where
    A: Node<C1f64> + ?Sized,
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

impl<A, DA> Node<C1f64> for Sine<A, DA>
where
    A: Node<C1f64> + ?Sized,
    DA: AsMut<A>,
{
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let f = self.frequency.as_mut().proc(ctx);
        let p = self.phase;
        self.phase = (self.phase.get_m() + f.get_m() / ctx.sample_rate as f64).fract().into();
        (p.get_m() * std::f64::consts::PI * 2.0).sin().into()
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
    A: Node<C1f64> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
