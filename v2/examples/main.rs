use std::f64::consts::TAU;

use corus::{
    nodes::{envelope::Envelope, param::Param, phase::Phase, sine::Sine, biquad_filter::BiquadFilter},
    EventQueue, ProccessContext,
};

fn main() {
    let mut ctx = ProccessContext::new(44100.0);
    let mut event_queue = EventQueue::new();
    let mut synth = Synth::new();

    let frequency_scheduler = synth.frequency.scheduler();
    frequency_scheduler.lock().unwrap().linear_ramp_to_value_at_time(0.5, 880.0);
    let gain_scheduler = synth.mod_gain.scheduler();
    gain_scheduler.lock().unwrap().set_value_at_time(0.0, 1.0);
    gain_scheduler.lock().unwrap().linear_ramp_to_value_at_time(0.25, 0.0);
    gain_scheduler.lock().unwrap().set_value_at_time(0.25, 1.0);
    gain_scheduler.lock().unwrap().linear_ramp_to_value_at_time(0.5, 0.0);
    gain_scheduler.lock().unwrap().set_value_at_time(0.5, 1.0);
    gain_scheduler.lock().unwrap().linear_ramp_to_value_at_time(0.75, 0.0);
    gain_scheduler.lock().unwrap().set_value_at_time(0.75, 1.0);
    gain_scheduler.lock().unwrap().linear_ramp_to_value_at_time(1.0, 0.0);

    frequency_scheduler.lock().unwrap().push_events(&mut event_queue, 0.0, 2.0);
    gain_scheduler.lock().unwrap().push_events(&mut event_queue, 0.0, 2.0);

    event_queue.push(0.0, synth.env.note_on_event(0.0));
    event_queue.push(1.0, synth.env.note_off_event(1.0));

    let name = "main.wav";
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: ctx.sample_rate().round() as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    for _ in 0..44100 * 2 {
        event_queue.dispatch(ctx.current_time());

        let x = synth.process(&ctx);
        let x = (x * std::i16::MAX as f64) as i16;
        writer.write_sample(x).unwrap();

        ctx.next();
    }
    writer.finalize().unwrap();
}

pub struct Synth {
    mod_phase: Phase,
    frequency: Param,
    phase: Phase,
    mod_gain: Param,
    env: Envelope,
    filter: BiquadFilter,
    filter_sin: Sine,
}

impl Synth {
    pub fn new() -> Self {
        Self {
            mod_phase: Phase::new(),
            frequency: Param::new(440.0),
            phase: Phase::new(),
            mod_gain: Param::new(0.5),
            env: Envelope::new(&[(0.1, 1.0, -1.0), (1.0, 0.5, 1.0)], 0.3, 1.0),
            filter: BiquadFilter::new(),
            filter_sin: Sine::new(),
        }
    }

    fn process(&mut self, ctx: &ProccessContext) -> f64 {
        let mod_phase = self.mod_phase.process(ctx, 40.0);
        let mod_gain = self.mod_gain.process(ctx);
        let modu = (mod_phase * TAU).sin() * mod_gain * 100.0;
        let f = self.frequency.process(ctx);
        let phase = self.phase.process(ctx, f + modu);
        let gain = self.env.process(ctx) * 0.2;
        let x = (phase * TAU).sin();
        let filter_freq = self.filter_sin.process(ctx, 5.0) * 500.0 + 1000.0;
        let x = self.filter.process(ctx, filter_freq, 3.0, x);
        x * gain
    }
}
