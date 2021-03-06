use crate::{
    core::{param3::ParamEventScheduleNode, Node},
    proc_context::ProcContext,
    signal::Mono,
};

use super::envelope2::EnvelopeGenerator;

type F = f64;

type Env<T> = (
    ParamEventScheduleNode<T>,
    Box<dyn FnMut(f64) + Send + Sync>,
    Box<dyn FnMut(f64) + Send + Sync>,
);

pub struct FmSynth<A, B>
where
    A: Node<F>,
    B: Node<F>,
{
    pub oscillators: [(A, B, Env<F>, F, [bool; 5], F); 4],
    pub frequency: ParamEventScheduleNode<F>,
}

impl<A, B> FmSynth<A, B>
where
    A: Node<F>,
    B: Node<F>,
{
    pub fn new<E: EnvelopeGenerator<F>>(oscillators: [(A, B, E, F, Vec<u8>); 4]) -> Self {
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
            frequency: ParamEventScheduleNode::new(),
        }
    }

    pub fn note_on(&mut self, time: f64, frequency: F) {
        self.frequency
            .get_scheduler()
            .lock()
            .unwrap()
            .set_value_at_time(time, frequency);
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

impl<A, B> Node<F> for FmSynth<A, B>
where
    A: Node<F>,
    B: Node<F>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> F {
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
        outputs[4]
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.frequency.lock(ctx);
        for oscillator in &mut self.oscillators {
            oscillator.0.lock(ctx);
            oscillator.1.lock(ctx);
            oscillator.2 .0.lock(ctx);
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
