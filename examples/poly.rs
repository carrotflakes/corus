mod write_to_file;

use corus::{
    contrib::{
        envelope2::AdsrEnvelope,
        generic_poly_synth::{PolySynth, Voice},
    },
    core::{
        accumulator::{Accumulator, SetValueAtTime},
        add::Add,
        amp::Amp,
        controllable::Controllable,
        param3::ParamEventScheduleNode,
        var::Var,
    },
    notenum_to_frequency,
    signal::C1f64,
    EventControlInplace, EventPusher,
};

fn main() {
    let sample_rate = 44100;

    let builder = || {
        let freq = ParamEventScheduleNode::new();
        let freq_ctl = freq.get_scheduler();
        let acc = Controllable::new(EventControlInplace::new(Accumulator::new(
            freq,
            C1f64::from(1.0),
        )));
        let mut acc_ctl = acc.controller();
        let saw = Add::new(acc, Var::from(-0.5));
        let (env, mut env_on, mut env_off) = AdsrEnvelope::<f64>::new(0.01, 0.5, 0.2, 0.3).build();
        let node = Amp::new(saw, env);
        Voice::new(
            node,
            Box::new(move |time, notenum| {
                freq_ctl
                    .lock()
                    .unwrap()
                    .set_value_at_time(time, notenum_to_frequency(notenum));
                acc_ctl
                    .lock()
                    .push_event(time, SetValueAtTime::new(C1f64::from(0.5)));
                env_on(time);
            }),
            Box::new(move |time, ()| {
                env_off(time);
            }),
        )
    };
    let mut synth = PolySynth::new(&builder, 10);

    synth.note_on(0.2, 60, 60);
    synth.note_off(0.4, 60, ());
    synth.note_on(0.4, 64, 64);
    synth.note_off(0.6, 64, ());
    synth.note_on(0.6, 67, 67);
    synth.note_off(0.8, 67, ());
    synth.note_on(0.8, 64, 64);
    synth.note_off(1.0, 64, ());
    synth.note_on(1.0, 60, 60);
    synth.note_off(1.6, 60, ());
    synth.note_on(1.1, 64, 64);
    synth.note_off(1.6, 64, ());
    synth.note_on(1.2, 67, 67);
    synth.note_off(1.6, 67, ());
    synth.note_on(1.3, 71, 71);
    synth.note_off(1.6, 71, ());

    let node = Amp::new(synth, Var::from(0.1));

    write_to_file::write_to_file("poly.wav", sample_rate, 3.0, node, None, None);
}
