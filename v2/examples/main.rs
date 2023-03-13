use std::f64::consts::TAU;

use corus_v2::{
    nodes::{
        biquad_filter::BiquadFilter,
        effects::{DelayFx, SchroederReverb},
        envelope::Envelope,
        mix::mix,
        param::Param,
        phase::Phase,
        sine::Sine,
        unison::Unison,
        voice_manager::VoiceManager,
    },
    signal::{IntoStereo, SignalExt, StereoF64},
    unsafe_wrapper::UnsafeWrapper,
    EventQueue, PackedEvent, ProccessContext, Producer,
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

    let mut poly_synth = UnsafeWrapper::new(PolySynth::new());
    event_queue.push(0.5, PolySynth::note_on_event(&poly_synth, 60));
    event_queue.push(0.6, PolySynth::note_off_event(&poly_synth, 60));
    event_queue.push(0.6, PolySynth::note_on_event(&poly_synth, 64));
    event_queue.push(0.7, PolySynth::note_off_event(&poly_synth, 64));
    event_queue.push(0.7, PolySynth::note_on_event(&poly_synth, 67));
    event_queue.push(0.9, PolySynth::note_off_event(&poly_synth, 67));

    let mut delay_fx = DelayFx::new(44100);
    let mut reverb = SchroederReverb::new(44100);

    let name = "main.wav";
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: ctx.sample_rate().round() as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    for _ in 0..44100 * 3 {
        event_queue.dispatch(ctx.current_time());

        let x = synth
            .process(&ctx)
            .into_stereo()
            .add(poly_synth.process(&ctx));
        let x = delay_fx.process(&ctx, x, 0.5, 0.25);
        let x = mix(&[(0.9, x), (0.3, reverb.process(&ctx, x))]);
        let [l, r] = x.into_stereo_with_pan(0.0);
        writer
            .write_sample((l * std::i16::MAX as f64) as i16)
            .unwrap();
        writer
            .write_sample((r * std::i16::MAX as f64) as i16)
            .unwrap();

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
    filter: BiquadFilter<1, f64>,
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
    unison: Unison,
    envs: [Envelope; 2],
    filter: BiquadFilter<2, StereoF64>,
}

impl Voice {
    fn process(&mut self, ctx: &ProccessContext) -> StereoF64 {
        let x = self
            .unison
            .process(ctx, self.frequency, 0.04, 0.9, |phase| phase * 2.0 - 1.0);
        let gain = self.envs[0].process(ctx) * 0.4;
        let filter_freq = self.envs[1].process(ctx) * 4000.0 + 4500.0;
        let x = self.filter.process(ctx, filter_freq, 1.5, x);
        x.mul(gain.into_stereo())
    }

    fn note_on(&mut self, time: f64, payload: u8) {
        self.frequency = 440.0 * 2.0f64.powf((payload as f64 - 69.0) / 12.0);
        self.envs[0].note_on(time);
        self.envs[1].note_on(time);
        self.unison.reset();
    }

    fn note_off(&mut self, time: f64) {
        self.envs[0].note_off(time);
        self.envs[1].note_off(time);
    }
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            frequency: 440.0,
            unison: Unison::new(5),
            envs: [
                Envelope::new(&[(0.01, 1.0, -1.0), (2.0, 0.8, 1.0)], 0.3, 1.0),
                Envelope::new(&[(0.01, 1.0, -1.0), (1.0, 0.5, 1.0)], 0.3, 1.0),
            ],
            filter: BiquadFilter::new(),
        }
    }
}

struct PolySynth {
    voices: VoiceManager<u8, Voice>,
}

impl PolySynth {
    fn new() -> Self {
        Self {
            voices: VoiceManager::new(8),
        }
    }

    fn process(&mut self, ctx: &ProccessContext) -> StereoF64 {
        let mut x = StereoF64::default();
        for voice in self.voices.iter_mut() {
            x = x.add(voice.process(ctx));
        }
        x
    }

    fn note_on_event(this: &UnsafeWrapper<Self>, notenum: u8) -> PackedEvent {
        let mut this = this.clone();
        Box::new(move |time| {
            this.voices.note_on(notenum).note_on(time, notenum);
        })
    }

    fn note_off_event(this: &UnsafeWrapper<Self>, notenum: u8) -> PackedEvent {
        let mut this = this.clone();
        Box::new(move |time| {
            if let Some(voice) = this.voices.note_off(notenum) {
                voice.note_off(time);
            }
        })
    }
}
