use std::marker::PhantomData;

use crate::{
    node::{controllable::Controllable, param::Param, Node},
    proc_context::ProcContext,
    signal::Mono,
};

use super::envelope::EnvelopeGenerator;

type F = f64;

type Env<T: Mono<F>> = (
    Controllable<T, Param<F, T>>,
    Box<dyn FnMut(f64)>,
    Box<dyn FnMut(f64)>,
);

pub struct FmSynth<T, A, B, DA, DB>
where
    T: Mono<F>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    pub oscillators: [(DA, DB, Env<T>, F, [bool; 5], F); 4],
    pub frequency: Param<F, T>,
    _t: (PhantomData<A>, PhantomData<B>),
}

impl<T, A, B, DA, DB> FmSynth<T, A, B, DA, DB>
where
    T: Mono<F>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    pub fn new<E: EnvelopeGenerator<T>>(oscillators: [(DA, DB, E, F, Vec<u8>); 4]) -> Self {
        let f = |x: (DA, DB, E, F, Vec<u8>)| {
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
                Default::default(),
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

    pub fn note_on(&mut self, time: f64, frequency: F) {
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

impl<T, A, B, DA, DB> Node<T> for FmSynth<T, A, B, DA, DB>
where
    T: Mono<f64>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let mut outputs = [0.0; 5];
        let dtime = 1.0 / ctx.sample_rate as f64;
        let frequency = self.frequency.as_mut().proc(ctx).get_m();
        for (i, oscillator) in self.oscillators.iter_mut().enumerate() {
            let freq_rate = oscillator.0.as_mut().proc(ctx).get_m();
            let freq_tune = oscillator.1.as_mut().proc(ctx).get_m();
            let env = oscillator.2 .0.as_mut().proc(ctx).get_m();
            oscillator.5 = oscillator.5 + (frequency * freq_rate + freq_tune + outputs[i]) * dtime;
            oscillator.5 = oscillator.5.fract();
            let v = (oscillator.5 * std::f64::consts::PI * 2.0).sin() * oscillator.3 * env;
            for (i, b) in oscillator.4.iter().enumerate() {
                if *b {
                    outputs[i] += v;
                }
            }
        }
        T::from_m(outputs[4])
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

impl<T, A, B, DA, DB> AsMut<Self> for FmSynth<T, A, B, DA, DB>
where
    T: Mono<F>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}
