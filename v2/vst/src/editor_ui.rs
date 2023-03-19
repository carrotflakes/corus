use std::sync::Arc;

use crate::{synth::bender::Bender, MyPluginParams};
use nih_plug::prelude::*;
use nih_plug_egui::egui::{self, emath};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EnvelopeLocation {
    VoiceGain,
    VoiceEffector(usize, usize),
    MasterEffector(usize, usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EffectorsLocation {
    Voice,
    Master,
}

pub fn editor_updator(
    egui_ctx: &egui::Context,
    setter: &ParamSetter,
    state: &mut Arc<MyPluginParams>,
) {
    egui::CentralPanel::default().show(egui_ctx, |ui| {
        let mut synth = state.synth.lock().unwrap();

        ui.horizontal(|ui| {
            let wt = {
                let seed = synth.voice_params.wavetable_settings.seed;
                synth.voice_params.wavetable_settings.wt_cache.update(seed);
                synth
                    .voice_params
                    .wavetable_settings
                    .wt_cache
                    .get_ref(synth.voice_params.wavetable_settings.seed)
                    .unwrap()
                    .clone()
            };
            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                let (_id, rect) = ui.allocate_space(egui::vec2(80.0, 80.0));
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
                    egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                ));
                ui.painter().extend(shapes);
            });

            ui.vertical(|ui| {
                ui.add(egui::widgets::DragValue::new(
                    &mut synth.voice_params.wavetable_settings.seed,
                ));

                ui.horizontal(|ui| {
                    egui::Frame::canvas(ui.style()).show(ui, |ui| {
                        let (_id, rect) = ui.allocate_space(egui::vec2(20.0, 20.0));
                        let to_screen = emath::RectTransform::from_to(
                            egui::Rect::from_x_y_ranges(0.0..=1.0, -1.0..=0.0),
                            rect,
                        );
                        let mut shapes = vec![];

                        let w = rect.width() as usize;
                        let mut points = vec![];
                        for i in 0..=w {
                            let p = i as f64 / w as f64;
                            let v = synth
                                .voice_params
                                .bender
                                .process(synth.voice_params.bend_level, p)
                                as f32;
                            points.push(to_screen * egui::pos2(p as f32, -v));
                        }
                        shapes.push(egui::Shape::line(
                            points,
                            egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                        ));
                        ui.painter().extend(shapes);
                    });

                    let r = synth.voice_params.bender.level_range();
                    if ui
                        .add(crate::widgets::knob::knob(
                            r.start..r.end,
                            &mut synth.voice_params.bend_level,
                        ))
                        .secondary_clicked()
                    {
                        synth.voice_params.bend_level = 0.0;
                    };

                    egui::ComboBox::from_label("bend")
                        .selected_text(format!("{:?}", &synth.voice_params.bender))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut synth.voice_params.bender,
                                Bender::None,
                                "none",
                            );
                            ui.selectable_value(
                                &mut synth.voice_params.bender,
                                Bender::Quadratic,
                                "quadratic",
                            );
                            ui.selectable_value(
                                &mut synth.voice_params.bender,
                                Bender::Cubic,
                                "cubic",
                            );
                            ui.selectable_value(&mut synth.voice_params.bender, Bender::Sin, "sin");
                            ui.selectable_value(&mut synth.voice_params.bender, Bender::Cos, "cos");
                        });
                });

                ui.checkbox(
                    &mut synth.voice_params.wavetable_settings.use_buffer,
                    "prerender",
                );
            });
        });
        ui.collapsing("Unison", |ui| {
            ui.horizontal(|ui| {
                ui.add(
                    egui::widgets::DragValue::new(&mut synth.voice_params.unison_settings.num)
                        .clamp_range(1..=10),
                );
                ui.label("voices");
                ui.checkbox(
                    &mut synth.voice_params.unison_settings.phase_reset,
                    "phase reset",
                );
            });
            ui.add(egui::widgets::Slider::new(
                &mut synth.voice_params.unison_settings.detune,
                0.0..=1.0,
            ));
            ui.add(egui::widgets::Slider::new(
                &mut synth.voice_params.unison_settings.stereo_width,
                0.0..=1.0,
            ));
        });

        ui.collapsing("Envelope", |ui| {
            ui.horizontal(|ui| {
                if ui.button("gain").clicked() {
                    *state.envelope_location.lock().unwrap() = EnvelopeLocation::VoiceGain;
                }
                if ui.button("filter freq").clicked() {
                    *state.envelope_location.lock().unwrap() =
                        EnvelopeLocation::VoiceEffector(0, 0);
                }
            });

            match state.envelope_location.lock().unwrap().clone() {
                EnvelopeLocation::VoiceGain => {
                    ui.label("Voice gain");
                    envelope(ui, &mut synth.voice_params.env);
                }
                EnvelopeLocation::VoiceEffector(i, j) => {
                    if let Some((_, fx)) = synth.voice_params.effectors.get_mut(i) {
                        ui.label(format!("V {} {}", fx.name(), fx.param_names()[j]));
                        if let Some(p) = fx.param_muts().get_mut(j) {
                            p.envelope.as_mut().map(|e| envelope(ui, &mut e.2));
                            p.lfo.as_mut().map(|l| lfo(ui, &mut l.1));
                        }
                    }
                }
                EnvelopeLocation::MasterEffector(i, j) => {
                    if let Some((_, fx)) = synth.effectors.get_mut(i) {
                        ui.label(format!("M {} {}", fx.name(), fx.param_names()[0]));
                        if let Some(p) = fx.param_muts().get_mut(j) {
                            p.envelope.as_mut().map(|e| envelope(ui, &mut e.2));
                            p.lfo.as_mut().map(|l| lfo(ui, &mut l.1));
                        }
                    }
                }
            }
        });

        ui.collapsing("Effectors", |ui| {
            ui.horizontal(|ui| {
                if ui.button("voice").clicked() {
                    *state.effectors_location.lock().unwrap() = EffectorsLocation::Voice;
                }
                if ui.button("master").clicked() {
                    *state.effectors_location.lock().unwrap() = EffectorsLocation::Master;
                }
            });

            match state.effectors_location.lock().unwrap().clone() {
                EffectorsLocation::Voice => {
                    effectors(
                        &mut synth.voice_params.effectors,
                        ui,
                        |i: usize, j: usize| {
                            *state.envelope_location.lock().unwrap() =
                                EnvelopeLocation::VoiceEffector(i, j);
                        },
                    );
                }
                EffectorsLocation::Master => {
                    effectors(&mut synth.effectors, ui, |i: usize, j: usize| {
                        *state.envelope_location.lock().unwrap() =
                            EnvelopeLocation::MasterEffector(i, j);
                    });
                }
            }
        });

        ui.label("Gain");
        ui.add(nih_plug_egui::widgets::ParamSlider::for_param(
            &state.gain,
            setter,
        ));

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
}

fn envelope(ui: &mut egui::Ui, envelope: &mut corus_v2::nodes::envelope::Envelope) {
    ui.horizontal(|ui| {
        ui.add(crate::widgets::knob::knob(
            0.0..1.0,
            &mut envelope.points[0].0,
        ));
        ui.add(crate::widgets::knob::knob(
            0.0..8.0,
            &mut envelope.points[1].0,
        ));
        ui.add(crate::widgets::knob::knob(
            0.0..1.0,
            &mut envelope.points[1].1,
        ));
        ui.add(crate::widgets::knob::knob(
            0.0..1.0,
            &mut envelope.release_length,
        ));
    });
}

fn lfo(ui: &mut egui::Ui, lfo: &mut crate::synth::param_f64::Lfo) {
    ui.horizontal(|ui| {
        ui.add(crate::widgets::knob::knob(0.001..100.0, &mut lfo.frequency));
        ui.add(crate::widgets::knob::knob(-10.0..10.0, &mut lfo.amp));
    });
}

fn effectors(
    effectors: &mut Vec<(bool, crate::synth::effectors::Effector)>,
    ui: &mut egui::Ui,
    setter: impl Fn(usize, usize),
) {
    for (i, (enabled, effector)) in effectors.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.add(egui::widgets::Checkbox::new(enabled, effector.name()));
            use crate::synth::effectors::Effector;

            match effector {
                Effector::Filter { frequency, q } => {
                    add_knob(ui, frequency, 20.0..10000.0, || setter(i, 0));
                    add_knob(ui, q, 0.7..10.0, || setter(i, 1));
                }
                Effector::Phaser => {}
                Effector::Chorus => {}
                Effector::Delay => {}
                Effector::Reverb => {}
                Effector::Gain { gain } => {
                    // ui.add(egui::widgets::Slider::new(gain, 0.0..=1.5));
                    add_knob(ui, gain, 0.0..1.5, || setter(i, 0));
                }
                Effector::Compressor {
                    threshold,
                    ratio,
                    attack,
                    release,
                    gain,
                } => {
                    add_knob(ui, threshold, 0.0..1.0, || setter(i, 0));
                    add_knob(ui, ratio, 0.0..1.0, || setter(i, 1));
                    add_knob(ui, attack, 0.001..1.0, || setter(i, 2));
                    add_knob(ui, release, 0.001..1.0, || setter(i, 3));
                    add_knob(ui, gain, 0.0..1.5, || setter(i, 4));
                }
                Effector::Tanh {} => {}
            }
        });
    }
}

fn add_knob(
    ui: &mut egui::Ui,
    param: &mut crate::synth::param_f64::ParamF64,
    range: std::ops::Range<f64>,
    on_click: impl Fn() -> (),
) {
    if ui
        .add(crate::widgets::knob::knob(range, &mut param.value))
        .clicked()
    {
        on_click();
    }
}
