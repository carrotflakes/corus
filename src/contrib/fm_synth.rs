use crate::{
    core::{controllable::Controllable, param::Param, Node},
    proc_context::ProcContext,
    signal::Mono,
};

use super::envelope::EnvelopeGenerator;

type F = f64;

type Env<T> = (
    Controllable<T, Param<F, T>>,
    Box<dyn FnMut(f64)>,
    Box<dyn FnMut(f64)>,
);

pub struct FmSynth<T, A, B>
where
    T: Mono<F>,
    A: Node<T>,
    B: Node<T>,
{
    pub oscillators: [(A, B, Env<T>, F, [bool; 5], F); 4],
    pub frequency: Param<F, T>,
}

impl<T, A, B> FmSynth<T, A, B>
where
    T: Mono<F>,
    A: Node<T>,
    B: Node<T>,
{
    pub fn new<E: EnvelopeGenerator<T>>(oscillators: [(A, B, E, F, Vec<u8>); 4]) -> Self {
        let f = |x: (A, B, E, F, Vec<u8>)| {
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

impl<T, A, B> Node<T> for FmSynth<T, A, B>
where
    T: Mono<f64>,
    A: Node<T>,
    B: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let mut outputs = [0.0; 5];
        let dtime = 1.0 / ctx.sample_rate as f64;
        let frequency = self.frequency.proc(ctx).get_m();
        for (i, oscillator) in self.oscillators.iter_mut().enumerate() {
            let freq_rate = oscillator.0.proc(ctx).get_m();
            let freq_tune = oscillator.1.proc(ctx).get_m();
            let env = oscillator.2 .0.proc(ctx).get_m();
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
        self.frequency.lock();
        for oscillator in &mut self.oscillators {
            oscillator.0.lock();
            oscillator.1.lock();
            oscillator.2 .0.lock();
        }
    }

    fn unlock(&mut self) {
        self.frequency.unlock();
        for oscillator in &mut self.oscillators {
            oscillator.0.unlock();
            oscillator.1.unlock();
            oscillator.2 .0.unlock();
        }
    }
}
