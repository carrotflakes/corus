mod synth;

use corus_v2::{event_queue::EventQueue, signal::Stereo};
use nih_plug::prelude::*;
use nih_plug_egui::{
    create_egui_editor,
    egui::{self, emath},
    widgets, EguiState,
};
use std::sync::{Arc, Mutex};
use synth::{MyEvent, MySynth};

struct MyPlugin {
    params: Arc<MyPluginParams>,
    context: corus_v2::ProccessContext,
    event_queue: EventQueue<MyEvent>,
}

#[derive(Params)]
struct MyPluginParams {
    #[persist = "editor-state"]
    editor_state: Arc<EguiState>,
    synth: Arc<Mutex<MySynth>>,

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
            context: corus_v2::ProccessContext::new(44100.0),
            event_queue: EventQueue::new(),
        }
    }
}

impl Default for MyPluginParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(300, 450),
            synth: Arc::new(Mutex::new(MySynth::new())),
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

    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        // let peak_meter = self.peak_meter.clone();
        create_egui_editor(
            self.params.editor_state.clone(),
            (),
            |_, _| {},
            move |egui_ctx, setter, _state| {
                egui::CentralPanel::default().show(egui_ctx, |ui| {
                    let mut synth = params.synth.lock().unwrap();

                    let wt = {
                        let seed = synth.voice_params.seed;
                        synth.voice_params.wt_cache.update(seed);
                        synth
                            .voice_params
                            .wt_cache
                            .get_ref(synth.voice_params.seed)
                            .unwrap()
                            .clone()
                    };
                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        let (_id, rect) = ui.allocate_space(egui::vec2(100.0, 100.0));
                        let to_screen = emath::RectTransform::from_to(
                            egui::Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                            rect,
                        );
                        let mut shapes = vec![];

                        let w = rect.width() as usize;
                        let mut points = vec![];
                        for i in 0..=w {
                            let p = i as f64 / w as f64;
                            let v = wt(p % 1.0) as f32;
                            points.push(to_screen * egui::pos2(p as f32, -v));
                        }
                        shapes.push(egui::Shape::line(
                            points,
                            egui::Stroke::new(1.0, egui::Color32::RED),
                        ));
                        ui.painter().extend(shapes);
                    });

                    ui.add(egui::widgets::DragValue::new(&mut synth.voice_params.seed));
                    ui.add(egui::widgets::Checkbox::new(
                        &mut synth.delay_enabled,
                        "Delay",
                    ));
                    ui.add(egui::widgets::Checkbox::new(
                        &mut synth.global_filter_enabled,
                        "Global filter",
                    ));
                    ui.add(egui::widgets::Checkbox::new(
                        &mut synth.voice_params.filter_enabled,
                        "Filter",
                    ));
                    ui.add(egui::widgets::Checkbox::new(
                        &mut synth.phaser_enabled,
                        "Phaser",
                    ));
                    ui.add(egui::widgets::Checkbox::new(
                        &mut synth.chorus_enabled,
                        "Chorus",
                    ));
                    ui.collapsing("Unison", |ui| {
                        ui.horizontal(|ui| {
                            ui.add(
                                egui::widgets::DragValue::new(&mut synth.unison_num).clamp_range(1..=10),
                            );
                            ui.label("voices");
                        });
                        ui.add(egui::widgets::Slider::new(
                            &mut synth.voice_params.detune,
                            0.0..=1.0,
                        ));
                        ui.add(egui::widgets::Slider::new(
                            &mut synth.voice_params.stereo_width,
                            0.0..=1.0,
                        ));
                    });

                    ui.collapsing("ADSR envelope", |ui| {
                        ui.add(egui::widgets::Slider::new(
                            &mut synth.voice_params.env.points[0].0,
                            0.0..=1.0,
                        ));
                        ui.add(egui::widgets::Slider::new(
                            &mut synth.voice_params.env.points[1].0,
                            0.0..=8.0,
                        ));
                        ui.add(egui::widgets::Slider::new(
                            &mut synth.voice_params.env.points[1].1,
                            0.0..=1.0,
                        ));
                        ui.add(egui::widgets::Slider::new(
                            &mut synth.voice_params.env.release_length,
                            0.0..=1.0,
                        ));
                    });

                    ui.label("Gain");
                    ui.add(widgets::ParamSlider::for_param(&params.gain, setter));

                    // ui.add(
                    //     egui::widgets::Slider::from_get_set(-30.0..=30.0, |new_value| {
                    //         match new_value {
                    //             Some(new_value_db) => {
                    //                 let new_value = util::gain_to_db(new_value_db as f32);

                    //                 setter.begin_set_parameter(&params.gain);
                    //                 setter.set_parameter(&params.gain, new_value);
                    //                 setter.end_set_parameter(&params.gain);

                    //                 new_value_db
                    //             }
                    //             None => util::gain_to_db(params.gain.value()) as f64,
                    //         }
                    //     })
                    //     .suffix(" dB"),
                    // );

                    // // TODO: Add a proper custom widget instead of reusing a progress bar
                    // let peak_meter =
                    //     util::gain_to_db(peak_meter.load(std::sync::atomic::Ordering::Relaxed));
                    // let peak_meter_text = if peak_meter > util::MINUS_INFINITY_DB {
                    //     format!("{peak_meter:.1} dBFS")
                    // } else {
                    //     String::from("-inf dBFS")
                    // };

                    // let peak_meter_normalized = (peak_meter + 60.0) / 60.0;
                    // ui.allocate_space(egui::Vec2::splat(2.0));
                    // ui.add(
                    //     egui::widgets::ProgressBar::new(peak_meter_normalized)
                    //         .text(peak_meter_text),
                    // );
                });
            },
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
            self.context = corus_v2::ProccessContext::new(sample_rate);
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
                            let value = value as f64 / 12.0;
                            self.event_queue.push(time, MyEvent::SetModLevel(value));
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

        // apply params
        synth.frequency = self.params.frequency.value() as f64;
        synth.q = self.params.resonance.value() as f64;

        for mut channel_samples in buffer.iter_samples() {
            self.event_queue
                .dispatch(self.context.current_time(), |_eq, time, event| {
                    synth.handle_event(event, time)
                });
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.gain.smoothed.next();
            let x = synth.process(&self.context);

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
