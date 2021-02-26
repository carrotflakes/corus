mod write_to_file;

use corus::{
    contrib::{
        chip::Noise,
        controllable_param,
        envelope::{ArEnvelope, EnvelopeGenerator},
        retriggerable_sine,
    },
    core::{amp::Amp, mix::Mix},
    signal::C1f64,
};

use corus::core::{constant::Constant, Node};

const SAMPLE_RATE: usize = 44100;

fn main() {
    let bps = 2.0;

    let kick = {
        let (freq, mut freq_ctrl) = controllable_param(0.0);
        let (env, mut env_ctrl) = controllable_param(0.0);
        let (sine, mut sin_trig) = retriggerable_sine(freq);
        let node = Amp::new(Box::new(sine), env);
        let mut note_on = |time: f64| {
            sin_trig(time);
            freq_ctrl.lock().cancel_and_hold_at_time(time);
            freq_ctrl.lock().set_value_at_time(time, 400.0);
            freq_ctrl
                .lock()
                .exponential_ramp_to_value_at_time(time + 0.4, 20.0);
            env_ctrl.lock().cancel_and_hold_at_time(time);
            env_ctrl.lock().set_value_at_time(time, 1.0);
            env_ctrl
                .lock()
                .exponential_ramp_to_value_at_time(time + 0.3, 0.01);
        };
        note_on(0.0 / bps);
        note_on(1.0 / bps);
        note_on(2.0 / bps);
        note_on(3.0 / bps);

        Box::new(node) as Box<dyn Node<C1f64>>
    };

    let snare = {
        let mut noise = Noise::new();
        noise.short_freq = true;
        noise.freq = 5000;
        let (noise_env, mut note_on, _) = ArEnvelope::new(0.01, 0.2).generate();
        let node = Amp::new(noise, noise_env);
        // note_on(0.0 / bps);
        note_on(1.0 / bps);
        // note_on(2.0 / bps);
        note_on(3.0 / bps);
        Box::new(node) as Box<dyn Node<C1f64>>
    };

    let hh = {
        let mut noise = Noise::new();
        noise.freq = 12000;
        let (noise_env, mut note_on, _) = ArEnvelope::new(0.01, 0.2).generate();
        let node = Amp::new(noise, noise_env);
        note_on(0.5 / bps);
        note_on(1.5 / bps);
        note_on(2.5 / bps);
        note_on(3.5 / bps);
        Box::new(node) as Box<dyn Node<C1f64>>
    };

    let node = Mix::new(vec![kick, snare, hh]);
    let node = Amp::new(node, Constant::from(0.2));
    write_to_file::write_to_file("beats.wav", SAMPLE_RATE, 4.0 / bps, node);
}
