mod editor_ui;
mod synth;
mod widgets;

use corus_v2::{event_queue::EventQueue, signal::Stereo};
use editor_ui::{EffectorsLocation, EnvelopeLocation};
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, EguiState};
use std::sync::{Arc, Mutex};
use synth::{MyEvent, MySynth};

pub struct MyPlugin {
    params: Arc<MyPluginParams>,
    context: corus_v2::ProcessContext,
    event_queue: EventQueue<MyEvent>,
}

#[derive(Params)]
pub struct MyPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,
    #[persist = "synth"]
    synth: Arc<Mutex<MySynth>>,
    synth_state: Arc<Mutex<synth::State>>,
    envelope_location: Mutex<EnvelopeLocation>,
    effectors_location: Mutex<EffectorsLocation>,
    wavetable_lab: Mutex<widgets::wavetable_lab::WavetableLab>,

    #[id = "gain"]
    pub gain: FloatParam,
    #[id = "frequency"]
    pub frequency: FloatParam,
    #[id = "resonance"]
    pub resonance: FloatParam,
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(MyPluginParams::default()),
            context: corus_v2::ProcessContext::new(44100.0),
            event_queue: EventQueue::new(),
        }
    }
}

impl Default for MyPluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(400, 400),
            synth: Arc::new(Mutex::new(MySynth::new())),
            synth_state: Arc::new(Mutex::new(synth::State::new())),
            envelope_location: Mutex::new(EnvelopeLocation::VoiceGain),
            effectors_location: Mutex::new(EffectorsLocation::Master),
            wavetable_lab: Mutex::new(widgets::wavetable_lab::WavetableLab::new()),
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-30.0),
                    max: util::db_to_gain(30.0),
                    factor: FloatRange::gain_skew_factor(-30.0, 30.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
            frequency: FloatParam::new(
                "Frequency",
                5000.0,
                FloatRange::Skewed {
                    min: 20.0,
                    max: 20000.0,
                    factor: FloatRange::skew_factor(-1.0),
                },
            )
            .with_unit(" Hz")
            .with_value_to_string(formatters::v2s_f32_hz_then_khz(2))
            .with_string_to_value(formatters::s2v_f32_hz_then_khz()),
            resonance: FloatParam::new(
                "Resonance",
                1.0,
                FloatRange::Skewed {
                    min: 2.0f32.sqrt() / 2.0,
                    max: 10.0,
                    factor: FloatRange::skew_factor(1.0),
                },
            )
            .with_value_to_string(formatters::v2s_f32_rounded(2)),
        }
    }
}

impl Plugin for MyPlugin {
    const NAME: &'static str = "corus example";
    const VENDOR: &'static str = "author";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "author@example.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(0),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        // let peak_meter = self.peak_meter.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            self.params.clone(),
            |_, _| {},
            editor_ui::editor_updator,
        )
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sample_rate = context.transport().sample_rate as f64;
        if self.context.sample_rate() != sample_rate {
            self.context = corus_v2::ProcessContext::new(sample_rate);
        }
        while let Some(event) = context.next_event() {
            #[allow(unused_variables)]
            match event {
                NoteEvent::NoteOn {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {
                    if channel != 0 {
                        continue;
                    }
                    let time = self.context.current_time() + timing as f64 / sample_rate;
                    self.event_queue
                        .push(time, MyEvent::NoteOn(note, velocity as f64));
                }
                NoteEvent::NoteOff {
                    timing,
                    voice_id,
                    channel,
                    note,
                    velocity,
                } => {
                    if channel != 0 {
                        continue;
                    }
                    let time = self.context.current_time() + timing as f64 / sample_rate;
                    self.event_queue.push(time, MyEvent::NoteOff(note));
                }
                NoteEvent::PolyPressure {
                    timing,
                    voice_id,
                    channel,
                    note,
                    pressure,
                } => {} // = aftertouch
                NoteEvent::MidiChannelPressure {
                    timing,
                    channel,
                    pressure,
                } => {} // = channel aftertouch
                NoteEvent::MidiPitchBend {
                    timing,
                    channel,
                    value,
                } => {
                    if channel != 0 {
                        continue;
                    }
                    let time = self.context.current_time() + timing as f64 / sample_rate;
                    let value = 2.0f64.powf((value as f64 * 2.0 - 1.0) / 12.0);
                    self.params.synth.lock().unwrap().pitch = value;
                }
                NoteEvent::MidiCC {
                    timing,
                    channel,
                    cc,
                    value,
                } => {
                    let time = self.context.current_time() + timing as f64 / sample_rate;
                    match cc {
                        control_change::MODULATION_MSB => {
                            // let value = value as f64 / 12.0;
                            // self.event_queue.push(time, MyEvent::SetModLevel(value));
                        }
                        //     control_change::SOUND_CONTROLLER_5 => {
                        //         // cutoff
                        //         self.synth
                        //             .filter_freq_ctl
                        //             .lock()
                        //             .unwrap()
                        //             .set_value_at_time(time, (value * 8000.0 + 10.0) as f64);
                        //     }
                        //     control_change::SOUND_CONTROLLER_2 => {
                        //         // resonance
                        //         self.synth
                        //             .filter_q_ctl
                        //             .lock()
                        //             .unwrap()
                        //             .set_value_at_time(time, (value * 50.0 + 0.5) as f64);
                        //     }
                        //     control_change::MAIN_VOLUME_MSB => {
                        //         self.synth
                        //             .gain_ctl
                        //             .lock()
                        //             .unwrap()
                        //             .set_value_at_time(time, value as f64);
                        //     }
                        //     control_change::PAN_MSB => {
                        //         self.synth
                        //             .pan_ctl
                        //             .lock()
                        //             .unwrap()
                        //             .set_value_at_time(time, (value * 2.0 - 1.0) as f64);
                        //     }
                        _ => {}
                    }
                }
                NoteEvent::MidiProgramChange {
                    timing,
                    channel,
                    program,
                } => {
                    // self.params.synth.lock().unwrap()
                    //     .handle_event(MyEvent::ProgramChange(program), self.context.current_time());
                }
                // NoteEvent::MidiSysEx { timing, message } => todo!(),
                _ => {}
            }
        }

        let mut synth = self.params.synth.lock().unwrap();
        let mut synth_state = self.params.synth_state.lock().unwrap();
        synth.ensure_state(&mut synth_state);

        // apply params
        synth.frequency = self.params.frequency.value() as f64;
        synth.q = self.params.resonance.value() as f64;

        for mut channel_samples in buffer.iter_samples() {
            self.event_queue
                .dispatch(self.context.current_time(), |_eq, time, event| {
                    synth.handle_event(&mut synth_state, event, time)
                });
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();
            let x = synth.process(&mut synth_state, &self.context);

            *channel_samples.get_mut(0).unwrap() = gain * x.get_l() as f32;
            *channel_samples.get_mut(1).unwrap() = gain * x.get_r() as f32;

            self.context.next();
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "corus example";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("corus example");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Synthesizer, ClapFeature::Stereo];
}

impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"corus example...";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[
        Vst3SubCategory::Instrument,
        Vst3SubCategory::Synth,
        Vst3SubCategory::Stereo,
    ];
}

// nih_export_clap!(MyPlugin);
nih_export_vst3!(MyPlugin);
