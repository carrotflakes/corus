use std::sync::{Arc, Mutex};

use corus::{
    contrib::{
        amp_pan,
        envelope2::AdsrEnvelope,
        generic_poly_synth::{PolySynth, Voice},
    },
    core::{
        accumulator::{Accumulator, SetValueAtTime},
        add::Add,
        amp::Amp,
        biquad_filter,
        controllable::{Controllable, Controller},
        mul::Mul,
        param3::{ParamEventSchedule, ParamEventScheduleNode},
        share::Share,
        var::Var,
    },
    db_to_amp, notenum_to_frequency,
    signal::C2f64,
    EventControllable, EventPusher, EventScheduleNode, Node,
};

const DB_MIN: f64 = 24.0;
pub type MyVoice = Voice<Box<dyn Node<Output = f64> + Send + Sync>, (u8, f64), ()>;

pub struct MySynth {
    node: Box<dyn Node<Output = C2f64> + Send + Sync>,
    pub synth_ctl: Controller<PolySynth<(u8, f64), (), MyVoice, u8>>,
    pub gain_ctl: Arc<Mutex<ParamEventSchedule<f64>>>,
    pub pan_ctl: Arc<Mutex<ParamEventSchedule<f64>>>,
    // pub modulation_ctl: Arc<Mutex<ParamEventSchedule<f64>>>,
    pub filter_freq_ctl: Arc<Mutex<ParamEventSchedule<f64>>>,
    pub filter_q_ctl: Arc<Mutex<ParamEventSchedule<f64>>>,
    pub pitch_ctl: Arc<Mutex<ParamEventSchedule<f64>>>,
}

impl MySynth {
    pub fn new() -> Self {
        let (pitch, pitch_ctl) = controllable_param(1.0);
        let pitch = pitch;
        let (gain, gain_ctl) = controllable_param(1.0);
        let (pan, pan_ctl) = controllable_param(0.0);
        // let (modulation, modulation_ctl) = controllable_param(0.0);
        // let modulated_pitch = Share::new(Add::new(pitch.clone(), modulation));
        let modulated_pitch = Share::new(pitch);
        let synth = PolySynth::new(&|| saw_builder(modulated_pitch.clone()), 8);
        let synth = Controllable::new(synth);
        let synth_ctl = synth.controller();
        let (filter_freq, filter_freq_ctl) = controllable_param(5000.0);
        let (filter_q, filter_q_ctl) = controllable_param(1.0);

        let node = biquad_filter::BiquadFilter::new(
            synth,
            biquad_filter::BiquadFilterParams::new(
                biquad_filter::types::LowPass,
                filter_freq,
                Var::new(0.0),
                filter_q,
            ),
        );
        let node = amp_pan(node, gain, pan);
        Self {
            node: Box::new(node),
            synth_ctl,
            gain_ctl,
            pan_ctl,
            // modulation_ctl,
            filter_freq_ctl,
            filter_q_ctl,
            pitch_ctl,
        }
    }

    pub fn program_change(&mut self, program: u32) {
        // self.synth = PolySynth::new(&|| saw_builder(self.pitch.clone()), 8);
    }
}

impl Node for MySynth {
    type Output = C2f64;

    fn proc(&mut self, ctx: &corus::ProcContext) -> Self::Output {
        self.node.proc(ctx)
    }

    fn lock(&mut self, ctx: &corus::ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

fn saw_builder(pitch: Share<ParamEventScheduleNode<f64>>) -> MyVoice {
    let (freq, freq_ctl) = controllable_param(1.0);
    let (gain, gain_ctl) = controllable_param(1.0);

    let acc = EventScheduleNode::new(EventControllable::new(Accumulator::new(
        Mul::new(freq, pitch),
        1.0,
    )));
    let mut acc_ctl = acc.get_scheduler();
    let mut acc_reset = move |time: f64| acc_ctl.push_event(time, SetValueAtTime::new(0.5));

    let saw = Add::new(acc, Var::from(-0.5));
    let (env, mut env_on, mut env_off) = AdsrEnvelope::<f64>::new(0.01, 0.5, 0.2, 0.9).build();
    let node = Amp::new(saw, Amp::new(env, gain));
    Voice::new(
        Box::new(node) as Box<dyn Node<Output = f64> + Send + Sync>,
        Box::new(move |time, (notenum, velocity)| {
            freq_ctl
                .lock()
                .unwrap()
                .set_value_at_time(time, notenum_to_frequency(notenum));
            gain_ctl
                .lock()
                .unwrap()
                .set_value_at_time(time, db_to_amp((velocity - 1.0) * DB_MIN));
            acc_reset(time);
            env_on(time);
        }),
        Box::new(move |time, ()| env_off(time)),
    )
}

pub fn controllable_param(
    v: f64,
) -> (
    ParamEventScheduleNode<f64>,
    Arc<Mutex<ParamEventSchedule<f64>>>,
) {
    let c = ParamEventScheduleNode::from_value(v);
    let ctl = c.get_scheduler();
    (c, ctl)
}
