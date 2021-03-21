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

pub struct Operator<A, B>
where
    A: Node<F>,
    B: Node<F>,
{
    freq_rate: A,
    freq_tune: B,
    env: Env<F>,
    inputs: [F; 4],
    phase: F,
}

pub struct FmSynth<A, B>
where
    A: Node<F>,
    B: Node<F>,
{
    pub operators: [Operator<A, B>; 4],
    pub frequency: ParamEventScheduleNode<F>,
    values: [F; 4],
    outputs: [F; 4],
}

impl<A, B> FmSynth<A, B>
where
    A: Node<F>,
    B: Node<F>,
{
    pub fn new<E: EnvelopeGenerator<F>>(
        operators: [(A, B, E, [F; 4]); 4],
        outputs: [F; 4],
    ) -> Self {
        let f = |x: (A, B, E, [F; 4])| Operator {
            freq_rate: x.0,
            freq_tune: x.1,
            env: x.2.generate(),
            inputs: x.3,
            phase: Default::default(),
        };
        let [o0, o1, o2, o3] = operators;
        let operators = [f(o0), f(o1), f(o2), f(o3)];
        Self {
            operators,
            frequency: ParamEventScheduleNode::new(),
            values: Default::default(),
            outputs,
        }
    }

    pub fn note_on(&mut self, time: f64, frequency: F) {
        self.frequency
            .get_scheduler()
            .lock()
            .unwrap()
            .set_value_at_time(time, frequency);
        for operator in &mut self.operators {
            (operator.env.1)(time);
        }
    }

    pub fn note_off(&mut self, time: f64) {
        for operator in &mut self.operators {
            (operator.env.2)(time);
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
        let dtime = 1.0 / ctx.sample_rate as f64;
        let frequency = self.frequency.proc(ctx).get_m();
        for (i, operator) in self.operators.iter_mut().enumerate() {
            let input = operator.inputs[0] * self.values[0]
                + operator.inputs[1] * self.values[1]
                + operator.inputs[2] * self.values[2]
                + operator.inputs[3] * self.values[3];
            let freq_rate = operator.freq_rate.proc(ctx).get_m();
            let freq_tune = operator.freq_tune.proc(ctx).get_m();
            let env = operator.env.0.proc(ctx).get_m();
            operator.phase = operator.phase + (frequency * freq_rate + freq_tune + input) * dtime;
            operator.phase = operator.phase.fract();
            self.values[i] = (operator.phase * std::f64::consts::PI * 2.0).sin() * env;
        }
        self.outputs[0] * self.values[0]
            + self.outputs[1] * self.values[1]
            + self.outputs[2] * self.values[2]
            + self.outputs[3] * self.values[3]
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.frequency.lock(ctx);
        for operator in &mut self.operators {
            operator.freq_rate.lock(ctx);
            operator.freq_tune.lock(ctx);
            operator.env.0.lock(ctx);
        }
    }

    fn unlock(&mut self) {
        self.frequency.unlock();
        for operator in &mut self.operators {
            operator.freq_rate.unlock();
            operator.freq_tune.unlock();
            operator.env.0.unlock();
        }
    }
}
