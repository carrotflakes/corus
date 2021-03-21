mod write_to_file;

use corus::{
    contrib::generic_poly_synth::{NoteOff, NoteOn, PolySynth, Voice},
    core::{
        accumulator::{Accumulator, SetValueAtTime},
        add::Add,
        amp::Amp,
        constant::Constant,
        controllable::Controllable,
        param3::ParamEventScheduleNode,
    },
    notenum_to_frequency,
    signal::C1f64,
    EventControlInplace, EventPusher,
};

fn main() {
    let sample_rate = 44100;

    let mut builder = || {
        let mut freq = ParamEventScheduleNode::new();
        let freq_ctl = freq.get_scheduler();
        let acc = Controllable::new(EventControlInplace::new(Accumulator::new(
            freq,
            C1f64::from(1.0),
        )));
        let mut acc_ctl = acc.controller();
        let saw = Add::new(acc, Constant::from(-0.5));
        let mut env = ParamEventScheduleNode::new();
        let env_ctl = env.get_scheduler();
        let node = Amp::new(saw, env);
        Voice(
            node,
            Box::new({
                let env_ctl = env_ctl.clone();
                move |time, NoteOn(notenum)| {
                    freq_ctl
                        .lock()
                        .unwrap()
                        .set_value_at_time(time, notenum_to_frequency(notenum));
                    acc_ctl
                        .lock()
                        .push_event(time, SetValueAtTime::new(C1f64::from(0.5)));
                    let mut env = env_ctl.lock().unwrap();
                    env.cancel_and_hold_at_time(time);
                    env.set_value_at_time(time, 0.001);
                    env.exponential_ramp_to_value_at_time(time + 0.01, 1.0);
                    env.exponential_ramp_to_value_at_time(time + 0.2, 0.5);
                }
            }),
            Box::new({
                let env_ctl = env_ctl.clone();
                move |time, NoteOff(())| {
                    let mut env = env_ctl.lock().unwrap();
                    env.cancel_and_hold_at_time(time);
                    env.set_target_at_time(time, 0.0, 0.1);
                }
            }),
        )
    };
    let mut synth = PolySynth::new(&mut builder, 10);

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

    let node = Amp::new(synth, Constant::from(0.1));

    write_to_file::write_to_file("poly.wav", sample_rate, 3.0, node, None, None);
}
