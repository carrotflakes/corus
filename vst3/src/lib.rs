mod synth;

use corus::{signal::Stereo, time::Sample, ProcContext};
use nih_plug::prelude::*;
use std::sync::Arc;
use synth::MySynth;

struct MyPlugin {
    params: Arc<MyPluginParams>,
    synth: MySynth,
    context: ProcContext,
}

#[derive(Params)]
struct MyPluginParams {
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
            synth: MySynth::new(),
            context: ProcContext::new(44100),
        }
    }
}

impl Default for MyPluginParams {
    fn default() -> Self {
        Self {
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(3.0),
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

    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let sample_rate = context.transport().sample_rate as f64;
        self.context.sample_rate = context.transport().sample_rate as u64;
        while let Some(event) = context.next_event() {
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
                    let time = self.context.current_time + timing as f64 / sample_rate;
                    self.synth
                        .synth_ctl
                        .lock()
                        .note_on(time, note, (note, velocity as f64));
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
                    let time = self.context.current_time + timing as f64 / sample_rate;
                    self.synth.synth_ctl.lock().note_off(time, note, ());
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
                    let time = self.context.current_time + timing as f64 / sample_rate;
                    self.synth
                        .pitch_ctl
                        .lock()
                        .unwrap()
                        .set_value_at_time(time, 2.0f64.powf((value as f64 * 2.0 - 1.0) / 12.0));
                }
                NoteEvent::MidiCC {
                    timing,
                    channel,
                    cc,
                    value,
                } => {
                    let time = self.context.current_time + timing as f64 / sample_rate;
                    match cc {
                        control_change::MODULATION_MSB => {
                            // self.synth.pan_ctl.lock().unwrap().set_value_at_time(time, (value * 2.0 - 1.0) as f64);
                        }
                        control_change::SOUND_CONTROLLER_5 => {
                            // cutoff
                            self.synth
                                .filter_freq_ctl
                                .lock()
                                .unwrap()
                                .set_value_at_time(time, (value * 8000.0 + 10.0) as f64);
                        }
                        control_change::SOUND_CONTROLLER_2 => {
                            // resonance
                            self.synth
                                .filter_q_ctl
                                .lock()
                                .unwrap()
                                .set_value_at_time(time, (value * 50.0 + 0.5) as f64);
                        }
                        control_change::MAIN_VOLUME_MSB => {
                            self.synth
                                .gain_ctl
                                .lock()
                                .unwrap()
                                .set_value_at_time(time, value as f64);
                        }
                        control_change::PAN_MSB => {
                            self.synth
                                .pan_ctl
                                .lock()
                                .unwrap()
                                .set_value_at_time(time, (value * 2.0 - 1.0) as f64);
                        }
                        _ => {}
                    }
                }
                NoteEvent::MidiProgramChange {
                    timing,
                    channel,
                    program,
                } => {
                    self.synth.program_change(program as u32);
                }
                // NoteEvent::MidiSysEx { timing, message } => todo!(),
                _ => {}
            }
        }

        // apply params
        let time = self.context.current_time + buffer.samples() as f64 / sample_rate;
        self.synth
            .filter_freq_ctl
            .lock()
            .unwrap()
            .linear_ramp_to_value_at_time(time, self.params.frequency.value() as f64);
        self.synth
            .filter_q_ctl
            .lock()
            .unwrap()
            .linear_ramp_to_value_at_time(time, self.params.resonance.value() as f64);

        let len = buffer.samples();
        let mut p = self.context.lock(&mut self.synth, Sample(len as u64));
        for mut channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();
            let x = p.next().unwrap();

            *channel_samples.get_mut(0).unwrap() = gain * x.get_l() as f32;
            *channel_samples.get_mut(1).unwrap() = gain * x.get_r() as f32;
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
