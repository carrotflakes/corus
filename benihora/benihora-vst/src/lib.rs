mod voice_manager;

use benihora::BenihoraManaged;
use nih_plug::prelude::*;
use nih_plug_egui::{create_egui_editor, egui, EguiState};
use std::sync::{Arc, Mutex};
use voice_manager::VoiceManager;

const SOUND_SPEED: usize = 2;

struct Synth {
    time: f64,
    benihora: BenihoraManaged,
    voice_manager: VoiceManager,
}

impl Synth {
    pub fn new(sample_rate: f64) -> Self {
        Synth {
            time: 0.0,
            benihora: BenihoraManaged::new(SOUND_SPEED, sample_rate),
            voice_manager: VoiceManager::new(),
        }
    }

    pub fn handle_event(&mut self, time: f64, event: &NoteEvent<()>) {
        match event {
            NoteEvent::NoteOn {
                timing,
                voice_id,
                channel,
                note,
                velocity,
            } => {
                if (24..24 + 5).contains(note) {
                    let (index, diameter) = [
                        (27.2, 2.20), // i
                        (19.4, 3.43), // e
                        (12.9, 2.43), // a
                        (14.0, 2.09), // o
                        (22.8, 2.05), // u
                    ][*note as usize - 24];
                    self.benihora.benihora.tract.mouth.tongue = self
                        .benihora
                        .benihora
                        .tract
                        .mouth
                        .tongue_clamp(index, diameter);
                    self.benihora.benihora.tract.calculate_diameter();
                    return;
                }
                if (24 + 5..24 + 5 + 3).contains(note) {
                    let (index, diameter) =
                        [(25.0, 0.2), (30.0, 0.2), (41.0, 0.7)][*note as usize - (24 + 5)];
                    self.benihora.benihora.tract.mouth.other_constrictions =
                        vec![benihora::Constriction {
                            index,
                            diameter,
                            start_time: time,
                            end_time: None,
                        }];
                    self.benihora.benihora.tract.calculate_diameter();
                    return;
                }

                self.voice_manager.noteon(*note);
                if let Some(note) = self.voice_manager.get_voice() {
                    self.benihora
                        .frequency
                        .set(440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0));
                    self.benihora.set_tenseness(*velocity as f64);
                    self.benihora.sound = true;
                }
            }
            NoteEvent::NoteOff {
                timing,
                voice_id,
                channel,
                note,
                velocity,
            } => {
                if (24 + 5..24 + 5 + 3).contains(note) {
                    if let Some(c) = self
                        .benihora
                        .benihora
                        .tract
                        .mouth
                        .other_constrictions
                        .get_mut(0)
                    {
                        c.end_time = Some(time);
                    }
                    self.benihora.benihora.tract.calculate_diameter();
                    return;
                }

                self.voice_manager.noteoff(*note);
                if let Some(note) = self.voice_manager.get_voice() {
                    self.benihora
                        .frequency
                        .set(440.0 * 2.0f64.powf((note as f64 - 69.0) / 12.0));
                    self.benihora.sound = true;
                } else {
                    self.benihora.sound = false;
                }
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
            } => {}
            NoteEvent::MidiCC {
                timing,
                channel,
                cc,
                value,
            } => {}
            NoteEvent::MidiProgramChange {
                timing,
                channel,
                program,
            } => {}
            _ => {}
        }
    }
}

struct MyPlugin {
    params: Arc<MyPluginParams>,
}

#[derive(Params)]
struct MyPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,

    #[id = "gain"]
    pub gain: FloatParam,

    synth: Mutex<Synth>,
}

impl Default for MyPlugin {
    fn default() -> Self {
        Self {
            params: Arc::new(MyPluginParams::default()),
        }
    }
}

impl Default for MyPluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(300, 180),

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

            synth: Mutex::new(Synth::new(48000.0)),
        }
    }
}

impl Plugin for MyPlugin {
    const NAME: &'static str = "benihora";
    const VENDOR: &'static str = "carrotflakes";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "carrotflakes@gmail.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(0),
        main_output_channels: NonZeroU32::new(1),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    ui.label("Some random integer");
                });
            },
        )
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
        let mut synth = self.params.synth.lock().unwrap();

        let sample_rate = context.transport().sample_rate as f64;
        if synth.benihora.benihora.tract.sample_rate != sample_rate {
            synth.benihora = BenihoraManaged::new(SOUND_SPEED, sample_rate);
        }

        let mut count = 0;
        let mut event = context.next_event();

        for mut channel_samples in buffer.iter_samples() {
            let current_time = synth.time;

            while let Some(e) = event {
                if e.timing() <= count {
                    synth.handle_event(current_time, &e);
                    event = context.next_event();
                } else {
                    break;
                }
            }
            count += 1;

            *channel_samples.get_mut(0).unwrap() = synth.benihora.process(current_time) as f32;
            synth.time += 1.0 / sample_rate;
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for MyPlugin {
    const CLAP_ID: &'static str = "benihora";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for MyPlugin {
    const VST3_CLASS_ID: [u8; 16] = *b"benihora\0\0\0\0\0\0\0\0";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Instrument];
}

// nih_export_clap!(MyPlugin);
nih_export_vst3!(MyPlugin);
