use std::marker::PhantomData;

use crate::{
    node::{controllable::Controllable, param::Param, Node},
    proc_context::ProcContext,
    signal::C1f32,
};

use super::envelope::EnvelopeGenerator;

type Env = (
    Controllable<C1f32, Param>,
    Box<dyn FnMut(f64)>,
    Box<dyn FnMut(f64)>,
);

pub struct FmSynth<A, B, DA, DB>
where
    A: Node<C1f32> + ?Sized,
    B: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    pub oscillators: [(DA, DB, Env, f32, [bool; 5], f32); 4],
    pub frequency: Param,
    _t: (PhantomData<A>, PhantomData<B>),
}

impl<A, B, DA, DB> FmSynth<A, B, DA, DB>
where
    A: Node<C1f32> + ?Sized,
    B: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    pub fn new<E: EnvelopeGenerator>(oscillators: [(DA, DB, E, f32, Vec<u8>); 4]) -> Self {
        let f = |x: (DA, DB, E, f32, Vec<u8>)| {
            (
                x.0,
                x.1,
                x.2.generate(),
                x.3,
                [
                    x.4.contains(&0),
                    x.4.contains(&1),
                    x.4.contains(&2),
                    x.4.contains(&3),
                    x.4.contains(&4),
                ],
                0.0,
            )
        };
        let [o0, o1, o2, o3] = oscillators;
        let oscillators = [f(o0), f(o1), f(o2), f(o3)];
        Self {
            oscillators,
            frequency: Param::new(),
            _t: Default::default(),
        }
    }

    pub fn note_on(&mut self, time: f64, frequency: f32) {
        self.frequency.set_value_at_time(time, frequency);
        for oscillator in &mut self.oscillators {
            (oscillator.2 .1)(time);
        }
    }

    pub fn note_off(&mut self, time: f64) {
        for oscillator in &mut self.oscillators {
            (oscillator.2 .2)(time);
        }
    }
}

impl<A, B, DA, DB> Node<C1f32> for FmSynth<A, B, DA, DB>
where
    A: Node<C1f32> + ?Sized,
    B: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f32 {
        let mut outputs = [0.0; 5];
        let dtime = 1.0 / ctx.sample_rate as f32;
        let frequency = self.frequency.as_mut().proc(ctx).0[0];
        for (i, oscillator) in self.oscillators.iter_mut().enumerate() {
            let freq_rate = oscillator.0.as_mut().proc(ctx).0[0];
            let freq_tune = oscillator.1.as_mut().proc(ctx).0[0];
            let env = oscillator.2 .0.as_mut().proc(ctx).0[0];
            oscillator.5 += (frequency * freq_rate + freq_tune + outputs[i]) * dtime;
            oscillator.5 = oscillator.5.fract();
            let v = (oscillator.5 * std::f32::consts::PI * 2.0).sin() * oscillator.3 * env;
            for (i, b) in oscillator.4.iter().enumerate() {
                if *b {
                    outputs[i] += v;
                }
            }
        }
        outputs[4].into()
    }

    fn lock(&mut self) {
        self.frequency.as_mut().lock();
        for oscillator in &mut self.oscillators {
            oscillator.0.as_mut().lock();
            oscillator.1.as_mut().lock();
            oscillator.2 .0.as_mut().lock();
        }
    }

    fn unlock(&mut self) {
        self.frequency.as_mut().unlock();
        for oscillator in &mut self.oscillators {
            oscillator.0.as_mut().unlock();
            oscillator.1.as_mut().unlock();
            oscillator.2 .0.as_mut().unlock();
        }
    }
}

impl<A, B, DA, DB> AsMut<Self> for FmSynth<A, B, DA, DB>
where
    A: Node<C1f32> + ?Sized,
    B: Node<C1f32> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
