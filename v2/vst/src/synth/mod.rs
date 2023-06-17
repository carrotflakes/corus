pub mod bender;
pub mod effectors;
pub mod param_f64;
pub mod param_pool;
pub mod wavetable;

use std::sync::Arc;

use serde::{Deserialize, Serialize};

use corus_v2::{
    nodes::{
        envelope::Envelope, first_order_filter::HighPassFilter, phase::Phase, unison::Unison,
        voice_manager::VoiceManager,
    },
    signal::{IntoStereo, StereoF64},
    ProcessContext,
};

use effectors::Effector;
use param_f64::{EnvelopeState, ParamF64};
use wavetable::WavetableSettings;

use self::{
    effectors::ShaperType,
    param_f64::Lfo,
    param_pool::{ParamPool, ProducerId},
};

#[derive(Serialize, Deserialize)]
pub struct MySynth {
    gain: f64,
    pan: f64,
    pub pitch: f64,
    pub voice: Voice,
    pub effectors: Vec<(bool, Effector)>,
    pub lfos: Vec<Lfo>,
}

pub struct State {
    voices: VoiceManager<u8, VoiceState>,
    effectors: Vec<effectors::State>,
    params: ParamPool,
    lfos: Vec<Phase<f64>>,
}

impl State {
    pub fn new(synth: &MySynth) -> Self {
        let producers: Vec<_> = synth
            .lfos
            .iter()
            .enumerate()
            .map(|(i, _)| ProducerId::new(i))
            .collect();
        Self {
            voices: VoiceManager::new(|| VoiceState::new(), 8),
            effectors: vec![],
            params: ParamPool::new(&producers),
            lfos: synth.lfos.iter().map(|_| Phase::new()).collect(),
        }
    }

    pub fn voice_num(&self) -> usize {
        self.voices.voice_num()
    }

    pub fn set_voice_num(&mut self, voice_num: usize) {
        self.voices.set_voice_num(voice_num);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Voice {
    pub wavetable_settings: WavetableSettings,
    pub bender: bender::Bender,
    pub bend_level: ParamF64,
    pub unison_settings: UnisonSettings,
    pub level: ParamF64,
    pub detune: ParamF64,
    pub effectors: Vec<(bool, Effector)>,
    pub envs: Vec<Envelope>,
}

#[derive(Serialize, Deserialize)]
pub struct UnisonSettings {
    pub num: usize,
    pub detune: f64,
    pub stereo_width: f64,
    pub phase_reset: bool,
}

impl MySynth {
    pub fn new() -> Self {
        Self {
            gain: 1.0,
            pan: 0.0,
            pitch: 1.0,
            voice: Voice {
                wavetable_settings: WavetableSettings::new(1),
                bender: bender::Bender::None,
                bend_level: ParamF64::new(0.0),
                unison_settings: UnisonSettings {
                    num: 1,
                    detune: 0.02,
                    stereo_width: 0.95,
                    phase_reset: false,
                },
                level: ParamF64::new(0.7),
                detune: ParamF64::new(0.0),
                effectors: vec![
                    (
                        false,
                        Effector::Shaper {
                            pre_gain: ParamF64::new(1.0),
                            r#type: ShaperType::Tanh,
                        },
                    ),
                    (
                        false,
                        Effector::Filter {
                            frequency: ParamF64::new(5000.0),
                            q: ParamF64::new(1.0),
                        },
                    ),
                ],
                envs: vec![
                    Envelope::new(&[(0.01, 1.0, -1.0), (2.0, 0.8, 1.0)], 0.2, 1.0),
                    Envelope::new(&[(0.01, 1.0, -1.0), (2.0, 0.8, 1.0)], 0.2, 1.0),
                ],
            },
            effectors: vec![
                (
                    false,
                    Effector::Filter {
                        frequency: ParamF64::new(10000.0),
                        q: ParamF64::new(1.0),
                    },
                ),
                (
                    false,
                    Effector::Shaper {
                        pre_gain: ParamF64::new(1.0),
                        r#type: ShaperType::Tanh,
                    },
                ),
                (false, Effector::Phaser),
                (false, Effector::Chorus),
                (false, Effector::Delay),
                (false, Effector::Reverb),
                (
                    true,
                    Effector::Gain {
                        gain: ParamF64::new(1.0),
                    },
                ),
                (
                    true,
                    Effector::Compressor {
                        threshold: ParamF64::new(0.8),
                        ratio: ParamF64::new(0.5),
                        attack: ParamF64::new(0.01),
                        release: ParamF64::new(0.03),
                        gain: ParamF64::new(1.0),
                    },
                ),
            ],
            lfos: vec![
                Lfo {
                    frequency: 1.0,
                    amp: 1.0,
                },
                Lfo {
                    frequency: 1.0,
                    amp: 1.0,
                },
            ],
        }
    }

    pub fn process(&mut self, state: &mut State, ctx: &ProcessContext) -> StereoF64 {
        let pitch = self.pitch;
        for (i, lfo) in self.lfos.iter().enumerate() {
            state.params.set(ProducerId::new(i), {
                let phase = state.lfos[i].process(ctx, lfo.frequency);
                lfo.compute(phase)
            });
        }

        let mut x = StereoF64::default();
        for voice in state.voices.iter_mut() {
            x = x + self.voice.process(voice, ctx, &state.params, pitch);
        }
        for ((enabled, effector), e_state) in self.effectors.iter().zip(state.effectors.iter_mut())
        {
            if *enabled {
                x = effector.process(e_state, ctx, &[&state.params], x);
            }
        }
        x = (x * self.gain.into_stereo()).into_stereo_with_pan(self.pan);
        x
    }

    pub fn ensure_state(&mut self, state: &mut State) {
        state
            .effectors
            .resize_with(self.effectors.len(), || effectors::State::None);
        for ((_, effector), state) in self.effectors.iter().zip(state.effectors.iter_mut()) {
            effector.ensure_state(state);
        }

        for voice in state.voices.iter_mut() {
            voice.unison.set_voice_num(self.voice.unison_settings.num);
            voice
                .effector_states
                .resize_with(self.voice.effectors.len(), || effectors::State::None);
            for ((_, effector), state) in self
                .voice
                .effectors
                .iter()
                .zip(voice.effector_states.iter_mut())
            {
                effector.ensure_state(state);
            }

            voice.wt = self.voice.wavetable_settings.generator();
        }
    }

    pub fn handle_event(&self, state: &mut State, event: MyEvent, time: f64) {
        match event {
            MyEvent::NoteOn(notenum, velocity) => {
                let v = state.voices.note_on(notenum);
                v.frequency = 440.0 * 2.0f64.powf((notenum as f64 - 69.0) / 12.0);
                if self.voice.unison_settings.phase_reset {
                    v.unison.reset();
                }
                v.velocity = velocity;
                v.note_time = Some((time, f64::INFINITY));
            }
            MyEvent::NoteOff(notenum) => {
                if let Some(v) = state.voices.note_off(notenum) {
                    v.note_time.iter_mut().for_each(|x| x.1 = time);
                }
            }
        }
    }
}

pub enum MyEvent {
    NoteOn(u8, f64),
    NoteOff(u8),
}

pub struct VoiceState {
    frequency: f64,
    velocity: f64,
    unison: Unison,
    note_time: Option<(f64, f64)>,
    high_pass_filter: HighPassFilter<StereoF64>,
    effector_states: Vec<effectors::State>,
    params: ParamPool,
    wt: Arc<dyn Fn(f64, f64) -> f64 + Send + Sync>,
}

impl Default for VoiceState {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceState {
    pub fn new() -> Self {
        Self {
            frequency: 440.0,
            velocity: 0.0,
            unison: Unison::new(3),
            note_time: None,
            high_pass_filter: HighPassFilter::new(),
            effector_states: vec![],
            params: ParamPool::new(&[ProducerId::new(0), ProducerId::new(1)]),
            wt: Arc::new(|_, _| 0.0),
        }
    }
}

impl Voice {
    pub fn process(
        &self,
        state: &mut VoiceState,
        ctx: &ProcessContext,
        param_pool: &ParamPool,
        pitch: f64,
    ) -> StereoF64 {
        let env_state = if let Some((start_time, end_time)) = state.note_time {
            EnvelopeState {
                elapsed: ctx.current_time() - start_time,
                note_off_time: end_time - start_time,
            }
        } else {
            return StereoF64::default();
        };

        for i in 0..self.envs.len() {
            state.params.set(
                ProducerId::new(i),
                self.envs[i].compute(env_state.elapsed, env_state.note_off_time),
            );
        }

        let bend_amount = self.bend_level.compute(&[&param_pool, &state.params]);
        let mut x = state.unison.process_range(
            ctx,
            state.frequency * pitch,
            self.unison_settings.detune,
            self.unison_settings.stereo_width,
            |phase, next_phase| {
                (state.wt)(
                    self.bender.process(bend_amount, phase),
                    self.bender.process(bend_amount, next_phase % 1.0) + next_phase.floor(),
                )
            },
        );

        // DC offset cancel
        x = state.high_pass_filter.process(ctx, 0.999, x);

        for ((enabled, effector), e_state) in
            self.effectors.iter().zip(state.effector_states.iter_mut())
        {
            if *enabled {
                x = effector.process(e_state, ctx, &[&param_pool, &state.params], x);
            }
        }

        let level = self
            .level
            .compute(&[&param_pool, &state.params])
            .clamp(0.0, 1.0);
        let gain = state.velocity * level;

        let env = state.params.get(ProducerId::new(0));
        x * gain * env
    }
}
