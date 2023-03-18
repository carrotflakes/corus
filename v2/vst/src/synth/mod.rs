pub mod bender;
mod cache;
pub mod effectors;
pub mod param_f64;
pub mod wavetable;

use corus_v2::{
    nodes::{
        envelope::Envelope, first_order_filter::HighPassFilter, sine::Sine, unison::Unison,
        voice_manager::VoiceManager,
    },
    signal::{IntoStereo, StereoF64},
    ProcessContext,
};

use effectors::Effector;
use param_f64::{EnvelopeState, ParamF64};
use wavetable::WavetableSettings;

use self::param_f64::Lfo;

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

pub struct VoiceParams {
    pub wavetable_settings: WavetableSettings,
    pub bender: bender::Bender,
    pub bend_level: f64,
    pub unison_settings: UnisonSettings,
    pub level: ParamF64,
    pub effectors: Vec<(bool, Effector)>,
    pub env: Envelope,
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
                wavetable_settings: WavetableSettings::new(0),
                bender: bender::Bender::None,
                bend_level: 0.0,
                unison_settings: UnisonSettings {
                    num: 1,
                    detune: 0.02,
                    stereo_width: 0.95,
                    phase_reset: false,
                },
                level: ParamF64 {
                    value: 0.7,
                    envelope: Some((
                        true,
                        0.0,
                        Envelope::new(&[(0.01, 1.0, -1.0), (2.0, 0.8, 1.0)], 0.2, 1.0),
                    )),
                    lfo: Some((
                        false,
                        Lfo {
                            frequency: 1.0,
                            amp: 0.5,
                        },
                    )),
                },
                effectors: vec![(
                    false,
                    Effector::Filter {
                        frequency: ParamF64 {
                            value: 50.0,
                            envelope: Some((
                                true,
                                5000.0,
                                Envelope::new(&[(0.01, 1.0, -1.0), (0.4, 0.3, 1.0)], 0.3, 1.0),
                            )),
                            lfo: Some((true, Lfo { frequency: 1.0, amp: 1000.0 })),
                        },
                        q: ParamF64 {
                            value: 1.0,
                            envelope: None,
                            lfo: None,
                        },
                    },
                )],
                env: Envelope::new(&[(0.01, 1.0, -1.0), (2.0, 0.8, 1.0)], 0.2, 1.0),
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
                            lfo: None,
                        },
                        q: ParamF64 {
                            value: 1.0,
                            envelope: None,
                            lfo: None,
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
                            lfo: None,
                        },
                    },
                ),
                (
                    true,
                    Effector::Compressor {
                        threshold: ParamF64 {
                            value: 0.8,
                            envelope: None,
                            lfo: None,
                        },
                        ratio: ParamF64 {
                            value: 0.5,
                            envelope: None,
                            lfo: None,
                        },
                        attack: ParamF64 {
                            value: 0.01,
                            envelope: None,
                            lfo: None,
                        },
                        release: ParamF64 {
                            value: 0.03,
                            envelope: None,
                            lfo: None,
                        },
                        gain: ParamF64 {
                            value: 1.0,
                            envelope: None,
                            lfo: None,
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
    high_pass_filter: HighPassFilter<StereoF64>,
    effector_states: Vec<effectors::State>,
}

impl MyVoice {
    pub fn new() -> Self {
        Self {
            frequency: 440.0,
            velocity: 0.0,
            unison: Unison::new(3),
            note_time: None,
            high_pass_filter: HighPassFilter::new(),
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

        let wt = param.wavetable_settings.generator();
        let mut x = self.unison.process(
            ctx,
            self.frequency * pitch,
            param.unison_settings.detune,
            param.unison_settings.stereo_width,
            |phase| (wt)(param.bender.process(param.bend_level, phase)),
        );

        // DC offset cancel
        x = self.high_pass_filter.process(ctx, 0.999, x);

        for ((enabled, effector), state) in
            param.effectors.iter().zip(self.effector_states.iter_mut())
        {
            if *enabled {
                x = effector.process(state, ctx, &env_state, x);
            }
        }

        let level = param.level.compute(&env_state).clamp(0.0, 1.0);
        let gain = self.velocity * level;

        let env = param
            .env
            .compute(env_state.elapsed, env_state.note_off_time);

        x * gain * env
    }
}
