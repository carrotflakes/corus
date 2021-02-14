use super::{Node, ProcContext};

pub struct Sine<A, DA>
where
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
{
    frequency: DA,
    phase: f32,
    _a: std::marker::PhantomData<A>,
}

impl<A, DA> Sine<A, DA>
where
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(frequency: DA) -> Self {
        Sine {
            frequency,
            phase: 0.0,
            _a: Default::default(),
        }
    }
}

impl<A, DA> Node<f32> for Sine<A, DA>
where
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
{
    fn proc(&mut self, ctx: &ProcContext) -> f32 {
        let f = self.frequency.as_mut().proc(ctx);
        let p = self.phase;
        self.phase = (self.phase + f / ctx.sample_rate as f32).fract();
        (std::f32::consts::PI * 2.0 * p).sin()
    }
}

impl<A, DA> AsMut<Self> for Sine<A, DA>
where
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
