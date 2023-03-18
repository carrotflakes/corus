pub mod bender;
mod cache;
pub mod effectors;
pub mod param_f64;

use std::sync::Arc;

use corus_v2::{
    nodes::{envelope::Envelope, sine::Sine, unison::Unison, voice_manager::VoiceManager},
    signal::{IntoStereo, StereoF64},
    ProcessContext,
};

use self::{
    effectors::Effector,
    param_f64::{EnvelopeState, ParamF64},
};

pub struct MySynth {
    voices: VoiceManager<u8, MyVoice>,
    gain: f64,
    pan: f64,
    pub pitch: f64,
    pub frequency: f64,
    pub q: f64,
    pub voice_params: VoiceParams,
    mod_level: f64,
    mod_sine: Sine<f64>,
    pub effectors: Vec<(bool, Effector)>,
    pub effector_states: Vec<effectors::State>,
    // lfo: Vec<Oscillator>,
    global_params: Vec<f64>,
}

type WT = Arc<dyn Fn(f64) -> f64 + Send + Sync + 'static>;

pub struct VoiceParams {
    pub seed: u64,
    pub wt_cache: cache::Cache<u64, WT, fn(u64) -> WT>,
    pub bender: bender::Bender,
    pub bend_level: f64,
    pub unison_settings: UnisonSettings,
    pub env: Envelope,
    pub filter_env: Envelope,
    pub filter_enabled: bool,
    pub effectors: Vec<(bool, Effector)>,
}

pub struct UnisonSettings {
    pub num: usize,
    pub detune: f64,
    pub stereo_width: f64,
    pub phase_reset: bool,
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
                unison_settings: UnisonSettings {
                    num: 1,
                    detune: 0.02,
                    stereo_width: 0.95,
                    phase_reset: false,
                },
                env: Envelope::new(&[(0.01, 1.0, -1.0), (2.0, 0.8, 1.0)], 0.2, 1.0),
                filter_env: Envelope::new(&[(0.01, 1.0, -1.0), (0.4, 0.3, 1.0)], 0.3, 1.0),
                filter_enabled: false,
                effectors: vec![(
                    false,
                    Effector::Filter {
                        frequency: ParamF64 {
                            value: 50.0,
                            envelope: Some((
                                5000.0,
                                Envelope::new(&[(0.01, 1.0, -1.0), (0.4, 0.3, 1.0)], 0.3, 1.0),
                            )),
                        },
                        q: ParamF64 {
                            value: 1.0,
                            envelope: None,
                        },
                    },
                )],
            },
            mod_level: 0.0,
            mod_sine: Sine::new(),
            effectors: vec![
                (
                    false,
                    Effector::Filter {
                        frequency: ParamF64 {
                            value: 10000.0,
                            envelope: None,
                        },
                        q: ParamF64 {
                            value: 1.0,
                            envelope: None,
                        },
                    },
                ),
                (false, Effector::Tanh),
                (false, Effector::Phaser),
                (false, Effector::Chorus),
                (false, Effector::Delay),
                (false, Effector::Reverb),
                (
                    true,
                    Effector::Gain {
                        gain: ParamF64 {
                            value: 1.0,
                            envelope: None,
                        },
                    },
                ),
                (
                    true,
                    Effector::Compressor {
                        threshold: ParamF64 {
                            value: 0.8,
                            envelope: None,
                        },
                        ratio: ParamF64 {
                            value: 0.5,
                            envelope: None,
                        },
                        attack: ParamF64 {
                            value: 0.001,
                            envelope: None,
                        },
                        release: ParamF64 {
                            value: 0.02,
                            envelope: None,
                        },
                        gain: ParamF64 {
                            value: 1.0,
                            envelope: None,
                        },
                    },
                ),
            ],
            effector_states: vec![],
            global_params: vec![20.0, 1.5],
        }
    }

    pub fn process(&mut self, ctx: &ProcessContext) -> StereoF64 {
        let modu = self.mod_sine.process(ctx, 3.0) * self.mod_level;
        let pitch = self.pitch * modu.exp2();
        let env_state = EnvelopeState {
            elapsed: ctx.current_time(),
            note_off_time: f64::INFINITY,
        };
        let mut x = StereoF64::default();
        for voice in self.voices.iter_mut() {
            x = x + voice.process(ctx, &mut self.voice_params, pitch, &self.global_params);
        }
        for ((enabled, effector), state) in
            self.effectors.iter().zip(self.effector_states.iter_mut())
        {
            if *enabled {
                x = effector.process(state, ctx, &env_state, x);
            }
        }
        x = (x * self.gain.into_stereo()).into_stereo_with_pan(self.pan);
        x
    }

    pub fn ensure_state(&mut self) {
        self.effector_states
            .resize_with(self.effectors.len(), || effectors::State::None);
        for ((_, effector), state) in self
            .effectors
            .iter_mut()
            .zip(self.effector_states.iter_mut())
        {
            effector.ensure_state(state);
        }

        for voice in self.voices.iter_mut() {
            voice
                .unison
                .set_voice_num(self.voice_params.unison_settings.num);
            voice
                .effector_states
                .resize_with(self.voice_params.effectors.len(), || effectors::State::None);
            for ((_, effector), state) in self
                .voice_params
                .effectors
                .iter()
                .zip(voice.effector_states.iter_mut())
            {
                effector.ensure_state(state);
            }
        }
    }

    pub fn handle_event(&mut self, event: MyEvent, time: f64) {
        match event {
            MyEvent::NoteOn(notenum, velocity) => {
                let v = self.voices.note_on(notenum);
                v.frequency = 440.0 * 2.0f64.powf((notenum as f64 - 69.0) / 12.0);
                if self.voice_params.unison_settings.phase_reset {
                    v.unison.reset();
                }
                v.velocity = velocity;
                v.note_time = Some((time, f64::INFINITY));
            }
            MyEvent::NoteOff(notenum) => {
                if let Some(v) = self.voices.note_off(notenum) {
                    v.note_time.iter_mut().for_each(|x| x.1 = time);
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
    frequency: f64,
    velocity: f64,
    unison: Unison,
    note_time: Option<(f64, f64)>,
    effector_states: Vec<effectors::State>,
}

impl MyVoice {
    pub fn new() -> Self {
        Self {
            frequency: 440.0,
            velocity: 0.0,
            unison: Unison::new(3),
            note_time: None,
            effector_states: vec![],
        }
    }

    pub fn process(
        &mut self,
        ctx: &ProcessContext,
        param: &mut VoiceParams,
        pitch: f64,
        global_params: &[f64],
    ) -> StereoF64 {
        let env_state = if let Some((start_time, end_time)) = self.note_time {
            EnvelopeState {
                elapsed: ctx.current_time() - start_time,
                note_off_time: end_time - start_time,
            }
        } else {
            return StereoF64::default();
        };

        let env = param
            .env
            .compute(env_state.elapsed, env_state.note_off_time);
        let gain = self.velocity * env;
        let wt = param.wt_cache.get(param.seed).clone();
        let mut x = self.unison.process(
            ctx,
            self.frequency * pitch,
            param.unison_settings.detune,
            param.unison_settings.stereo_width,
            |phase| (wt)(param.bender.process(param.bend_level, phase)),
        );

        for ((enabled, effector), state) in
            param.effectors.iter().zip(self.effector_states.iter_mut())
        {
            if *enabled {
                x = effector.process(state, ctx, &env_state, x);
            }
        }

        x * gain
    }
}
