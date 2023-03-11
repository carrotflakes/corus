use std::f64::consts::TAU;

use corus::{
    nodes::{
        biquad_filter::BiquadFilter,
        effects::DelayFx,
        envelope::Envelope,
        param::Param,
        phase::Phase,
        poly_synth::{NoteHandler, PolySynth},
        sine::Sine,
    },
    unsafe_wrapper::UnsafeWrapper,
    EventQueue, ProccessContext, Producer,
};

fn main() {
    let mut ctx = ProccessContext::new(44100.0);
    let mut event_queue = EventQueue::new();
    let mut synth = Synth::new();

    let frequency_scheduler = synth.frequency.scheduler();
    frequency_scheduler
        .lock()
        .unwrap()
        .linear_ramp_to_value_at_time(0.5, 880.0);
    let gain_scheduler = synth.mod_gain.scheduler();
    gain_scheduler.lock().unwrap().set_value_at_time(0.0, 1.0);
    gain_scheduler
        .lock()
        .unwrap()
        .linear_ramp_to_value_at_time(0.25, 0.0);
    gain_scheduler.lock().unwrap().set_value_at_time(0.25, 1.0);
    gain_scheduler
        .lock()
        .unwrap()
        .linear_ramp_to_value_at_time(0.5, 0.0);
    gain_scheduler.lock().unwrap().set_value_at_time(0.5, 1.0);
    gain_scheduler
        .lock()
        .unwrap()
        .linear_ramp_to_value_at_time(0.75, 0.0);
    gain_scheduler.lock().unwrap().set_value_at_time(0.75, 1.0);
    gain_scheduler
        .lock()
        .unwrap()
        .linear_ramp_to_value_at_time(1.0, 0.0);

    frequency_scheduler
        .lock()
        .unwrap()
        .push_events(&mut event_queue, 0.0, 2.0);
    gain_scheduler
        .lock()
        .unwrap()
        .push_events(&mut event_queue, 0.0, 2.0);

    event_queue.push(0.0, synth.env.make_event(|env, time| env.note_on(time)));
    event_queue.push(1.0, synth.env.make_event(|env, time| env.note_off(time)));

    let mut poly_synth = UnsafeWrapper::new(PolySynth::new(Voice::new, 8));
    event_queue.push(0.0, PolySynth::note_on_event(&poly_synth, 60, 60));
    event_queue.push(0.1, PolySynth::note_off_event(&poly_synth, 60, ()));
    event_queue.push(0.1, PolySynth::note_on_event(&poly_synth, 62, 62));
    event_queue.push(0.2, PolySynth::note_off_event(&poly_synth, 62, ()));
    event_queue.push(0.2, PolySynth::note_on_event(&poly_synth, 64, 64));
    event_queue.push(0.3, PolySynth::note_off_event(&poly_synth, 64, ()));

    let mut delay_fx = DelayFx::new(44100);

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

        let x = synth.process(&ctx) + poly_synth.process(&ctx);
        let x = delay_fx.process(&ctx, x, 0.5, 0.5);
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
    env: UnsafeWrapper<Envelope>,
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
            env: UnsafeWrapper::new(Envelope::new(
                &[(0.1, 1.0, -1.0), (1.0, 0.5, 1.0)],
                0.3,
                1.0,
            )),
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

pub struct Voice {
    frequency: f64,
    phase: Phase,
    env: Envelope,
    filter: BiquadFilter,
}

impl Voice {
    pub fn new() -> Self {
        Self {
            frequency: 440.0,
            phase: Phase::new(),
            env: Envelope::new(&[(0.01, 1.0, -1.0), (1.0, 0.5, 1.0)], 0.3, 1.0),
            filter: BiquadFilter::new(),
        }
    }

    fn process(&mut self, ctx: &ProccessContext) -> f64 {
        let phase = self.phase.process(ctx, self.frequency);
        let gain = self.env.process(ctx) * 0.2;
        let x = phase * 2.0 - 1.0;
        let x = self.filter.process(ctx, 5000.0, 3.0, x);
        x * gain
    }
}

impl Producer for Voice {
    type Output = f64;

    fn process(&mut self, ctx: &ProccessContext) -> Self::Output {
        self.process(ctx)
    }
}

impl NoteHandler<u8, ()> for Voice {
    fn note_on(&mut self, time: f64, payload: u8) {
        self.frequency = 440.0 * 2.0f64.powf((payload as f64 - 69.0) / 12.0);
        self.env.note_on(time);
        self.phase.set(0.0);
    }

    fn note_off(&mut self, time: f64, _: ()) {
        self.env.note_off(time);
    }
}
