pub mod bender;
mod cache;

use std::sync::Arc;

use corus_v2::{
    nodes::{
        biquad_filter::BiquadFilter,
        effects::{chorus::Chorus, phaser::Phaser, DelayFx},
        envelope::{self, Envelope},
        sine::Sine,
        unison::Unison,
        voice_manager::VoiceManager,
    },
    signal::{IntoStereo, StereoF64},
    ProccessContext,
};

pub struct MySynth {
    voices: VoiceManager<u8, MyVoice>,
    gain: f64,
    pan: f64,
    pub pitch: f64,
    pub frequency: f64,
    pub q: f64,
    pub global_filter_enabled: bool,
    pub phaser_enabled: bool,
    pub chorus_enabled: bool,
    pub voice_params: VoiceParams,
    pub delay_enabled: bool,
    pub unison_num: usize,
    mod_level: f64,
    mod_sine: Sine<f64>,
    filter: BiquadFilter<2, StereoF64>,
    pharser: Phaser<StereoF64>,
    delay_fx: DelayFx<StereoF64>,
    chorus: Chorus<StereoF64>,
}

type WT = Arc<dyn Fn(f64) -> f64 + Send + Sync + 'static>;

pub struct VoiceParams {
    pub seed: u64,
    pub wt_cache: cache::Cache<u64, WT, fn(u64) -> WT>,
    pub bender: bender::Bender,
    pub bend_level: f64,
    pub detune: f64,
    pub stereo_width: f64,
    pub env: Envelope,
    pub filter_env: Envelope,
    pub filter_enabled: bool,
}

impl MySynth {
    pub fn new() -> Self {
        let voices = VoiceManager::new(|| MyVoice::new(), 8);
        Self {
            voices,
            gain: 1.0,
            pan: 0.0,
            pitch: 1.0,
            frequency: 1000.0,
            q: 1.0,
            global_filter_enabled: false,
            phaser_enabled: false,
            chorus_enabled: false,
            voice_params: VoiceParams {
                seed: 0,
                wt_cache: cache::Cache::new(|seed: u64| {
                    match seed {
                        0 => wavetables::tree::Tree::Sin.build(),
                        1 => wavetables::tree::Tree::Saw.build(),
                        2 => wavetables::tree::Tree::Triangle.build(),
                        3 => wavetables::tree::Tree::Square.build(),
                        4 => wavetables::tree::Tree::Pulse(wavetables::tree::Value::Constant(
                            3.0 / 4.0,
                        ))
                        .build(),
                        5 => wavetables::tree::Tree::Pulse(wavetables::tree::Value::Constant(
                            7.0 / 8.0,
                        ))
                        .build(),
                        _ => {
                            let mut rng: rand::rngs::StdRng =
                                rand::SeedableRng::seed_from_u64(seed);
                            rand_wt::Config {
                                least_depth: 1,
                                variable_num: 0,
                            }
                            .generate(&mut rng)
                            .build()
                        }
                    }
                    .into()
                }),
                bender: bender::Bender::None,
                bend_level: 0.0,
                detune: 0.02,
                stereo_width: 0.95,
                env: Envelope::new(&[(0.01, 1.0, -1.0), (2.0, 0.8, 1.0)], 0.2, 1.0),
                filter_env: Envelope::new(&[(0.01, 1.0, -1.0), (0.4, 0.3, 1.0)], 0.3, 1.0),
                filter_enabled: false,
            },
            delay_enabled: false,
            unison_num: 1,
            mod_level: 0.0,
            mod_sine: Sine::new(),
            filter: BiquadFilter::new(),
            pharser: Phaser::new(),
            delay_fx: DelayFx::new(48000),
            chorus: Chorus::new(),
        }
    }

    pub fn process(&mut self, ctx: &ProccessContext) -> StereoF64 {
        let modu = self.mod_sine.process(ctx, 3.0) * self.mod_level;
        let pitch = self.pitch * modu.exp2();
        let mut x = StereoF64::default();
        for voice in self.voices.iter_mut() {
            voice.unison.set_voice_num(self.unison_num);
            x = x + voice.process(ctx, &mut self.voice_params, pitch);
        }
        if self.global_filter_enabled {
            x = self.filter.process(ctx, self.frequency, self.q, x);
        }
        if self.phaser_enabled {
            x = self.pharser.process(ctx, x);
        }
        if self.chorus_enabled {
            x = self.chorus.process(ctx, 0.001, 0.001, x);
        }
        if self.delay_enabled {
            x = self.delay_fx.process(ctx, x, 0.5, 0.3, 0.2);
        }
        (x * self.gain.into_stereo()).into_stereo_with_pan(self.pan)
    }

    pub fn handle_event(&mut self, event: MyEvent, time: f64) {
        match event {
            MyEvent::NoteOn(notenum, velocity) => {
                let v = self.voices.note_on(notenum);
                v.frequency = 440.0 * 2.0f64.powf((notenum as f64 - 69.0) / 12.0);
                // v.unison.reset();
                v.gain = velocity;
                v.env.note_on(time);
                v.filter_env.note_on(time);
            }
            MyEvent::NoteOff(notenum) => {
                if let Some(v) = self.voices.note_off(notenum) {
                    v.env.note_off(&self.voice_params.env, time);
                    v.filter_env.note_off(&self.voice_params.filter_env, time);
                }
            }
            MyEvent::SetModLevel(level) => {
                self.mod_level = level;
            }
        }
    }
}

pub enum MyEvent {
    NoteOn(u8, f64),
    NoteOff(u8),
    SetModLevel(f64),
}

pub struct MyVoice {
    unison: Unison,
    frequency: f64,
    gain: f64,
    env: envelope::State,
    filter_env: envelope::State,
    filter: BiquadFilter<2, StereoF64>,
}

impl MyVoice {
    pub fn new() -> Self {
        Self {
            unison: Unison::new(3),
            frequency: 440.0,
            gain: 0.0,
            env: envelope::State::new(),
            filter_env: envelope::State::new(),
            filter: BiquadFilter::new(),
        }
    }

    pub fn process(
        &mut self,
        ctx: &ProccessContext,
        param: &mut VoiceParams,
        pitch: f64,
    ) -> StereoF64 {
        let env = self.env.process(&param.env, ctx);
        let gain = self.gain * env;
        let wt = param.wt_cache.get(param.seed).clone();
        let mut x = self.unison.process(
            ctx,
            self.frequency * pitch,
            param.detune,
            param.stereo_width,
            |phase| (wt)(param.bender.process(param.bend_level, phase)),
        );
        if param.filter_enabled {
            let filter_env = self.filter_env.process(&param.filter_env, ctx);
            x = self
                .filter
                .process(ctx, filter_env * 10000.0 + 20.0, 1.5, x);
        }
        x * gain
    }
}
