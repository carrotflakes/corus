mod write_to_file;

use corus::{
    contrib::poly_synth::{PolySynth, Voice},
    core::{
        accumulator::{Accumulator, SetValueAtTime},
        amp::Amp,
        constant::Constant,
        controllable::Controllable,
        param::Param,
    },
    notenum_to_frequency,
    signal::C1f64,
    EventControlInplace, EventPusher,
};

fn main() {
    let sample_rate = 44100;

    let builder = || {
        let freq_param = Controllable::new(Param::new());
        let mut freq_param_ctrl = freq_param.controller();
        let acc = Controllable::new(EventControlInplace::new(Accumulator::new(
            freq_param,
            C1f64::from(1.0),
        )));
        let mut acc_ctrl = acc.controller();
        let saw = corus::core::add::Add::new(acc, Constant::from(-0.5));
        let env = Controllable::new(Param::new());
        let env_ctrl = env.controller();
        let node = Amp::new(saw, env);
        Voice::new(
            node,
            Box::new({
                let mut env_ctrl = env_ctrl.clone();
                move |time, notenum| {
                    freq_param_ctrl
                        .lock()
                        .set_value_at_time(time, notenum_to_frequency(notenum as u32));
                    acc_ctrl
                        .lock()
                        .push_event(time, SetValueAtTime::new(C1f64::from(0.5)));
                    let mut env = env_ctrl.lock();
                    env.cancel_and_hold_at_time(time);
                    env.set_value_at_time(time, 0.001);
                    env.exponential_ramp_to_value_at_time(time + 0.01, 1.0);
                    env.exponential_ramp_to_value_at_time(time + 0.2, 0.5);
                }
            }),
            Box::new({
                let mut env_ctrl = env_ctrl.clone();
                move |time| {
                    let mut env = env_ctrl.lock();
                    env.cancel_and_hold_at_time(time);
                    env.set_target_at_time(time, 0.0, 0.1);
                }
            }),
        )
    };
    let mut synth = PolySynth::new(&builder, 10);
    synth.note_on(0.2, 60);
    synth.note_off(0.4, 60);
    synth.note_on(0.4, 64);
    synth.note_off(0.6, 64);
    synth.note_on(0.6, 67);
    synth.note_off(0.8, 67);
    synth.note_on(0.8, 64);
    synth.note_off(1.0, 64);
    synth.note_on(1.0, 60);
    synth.note_off(1.6, 60);
    synth.note_on(1.1, 64);
    synth.note_off(1.6, 64);
    synth.note_on(1.2, 67);
    synth.note_off(1.6, 67);
    synth.note_on(1.3, 71);
    synth.note_off(1.6, 71);

    let node = Amp::new(synth, Constant::from(0.1));

    write_to_file::write_to_file("poly.wav", sample_rate, 3.0, node);
}
