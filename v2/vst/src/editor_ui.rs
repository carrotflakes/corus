use std::sync::Arc;

use crate::{
    synth::{bender::Bender, param_pool::ProducerId},
    MyPluginParams,
};
use nih_plug::prelude::*;
use nih_plug_egui::egui::{self, emath};

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

        ui.collapsing("Generator", |ui| {
            ui.horizontal(|ui| {
                let wt = { synth.voice.wavetable_settings.wavetable() };
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
                    wavetable_seed(&mut synth, ui);

                    ui.horizontal(|ui| {
                        egui::Frame::canvas(ui.style()).show(ui, |ui| {
                            let (_id, rect) = ui.allocate_space(egui::vec2(20.0, 20.0));
                            let to_screen = emath::RectTransform::from_to(
                                egui::Rect::from_x_y_ranges(0.0..=1.0, -1.0..=0.0),
                                rect,
                            );

                            let w = rect.width() as usize;
                            let mut points = vec![];
                            for i in 0..=w {
                                let p = i as f64 / w as f64;
                                let v = synth.voice.bender.process(synth.voice.bend_level.value, p)
                                    as f32;
                                points.push(to_screen * egui::pos2(p as f32, -v));
                            }
                            ui.painter().add(egui::Shape::line(
                                points,
                                egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                            ));
                        });

                        let r = synth.voice.bender.level_range();
                        add_knob(ui, &mut synth.voice.bend_level, r, true, || ());

                        egui::ComboBox::from_label("bend")
                            .selected_text(format!("{:?}", &synth.voice.bender))
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut synth.voice.bender, Bender::None, "none");
                                ui.selectable_value(
                                    &mut synth.voice.bender,
                                    Bender::Quadratic,
                                    "quadratic",
                                );
                                ui.selectable_value(
                                    &mut synth.voice.bender,
                                    Bender::Cubic,
                                    "cubic",
                                );
                                ui.selectable_value(&mut synth.voice.bender, Bender::Sin, "sin");
                                ui.selectable_value(&mut synth.voice.bender, Bender::Cos, "cos");
                            });
                    });

                    ui.checkbox(&mut synth.voice.wavetable_settings.use_buffer, "prerender");
                });
            });

            if ui
                .add(crate::widgets::knob::knob(
                    0.0..1.0,
                    &mut synth.voice.level.value,
                ))
                .clicked()
            {
                // *state.envelope_location.lock().unwrap() = EnvelopeLocation::VoiceGain;
            };

            ui.collapsing("Unison", |ui| {
                ui.horizontal(|ui| {
                    ui.add(
                        egui::widgets::DragValue::new(&mut synth.voice.unison_settings.num)
                            .clamp_range(1..=10),
                    );
                    ui.label("voices");
                    ui.checkbox(&mut synth.voice.unison_settings.phase_reset, "phase reset");
                });
                ui.add(egui::widgets::Slider::new(
                    &mut synth.voice.unison_settings.detune,
                    0.0..=1.0,
                ));
                ui.add(egui::widgets::Slider::new(
                    &mut synth.voice.unison_settings.stereo_width,
                    0.0..=1.0,
                ));
            });
        });

        ui.collapsing("Envelope", |ui| {
            let mut envloc = state.envelope_location.lock().unwrap();
            ui.horizontal(|ui| {
                for i in 0..synth.voice.envs.len() {
                    if ui
                        .selectable_label(*envloc == i, format!("{}", i))
                        .clicked()
                    {
                        *envloc = i;
                    }
                }
            });

            envelope(ui, &mut synth.voice.envs[*envloc]);

            for l in synth.lfos.iter_mut() {
                lfo(ui, l);
            }
        });

        ui.collapsing("Effectors", |ui| {
            ui.horizontal(|ui| {
                for (name, location) in [
                    ("voice", EffectorsLocation::Voice),
                    ("master", EffectorsLocation::Master),
                ] {
                    let mut loc = state.effectors_location.lock().unwrap();
                    if ui.selectable_label(*loc == location, name).clicked() {
                        *loc = location;
                    }
                }
            });

            match state.effectors_location.lock().unwrap().clone() {
                EffectorsLocation::Voice => {
                    effectors(
                        &mut synth.voice.effectors,
                        ui,
                        true,
                        |i: usize, j: usize| {
                            // *state.envelope_location.lock().unwrap() =
                            //     EnvelopeLocation::VoiceEffector(i, j);
                        },
                    );
                }
                EffectorsLocation::Master => {
                    effectors(&mut synth.effectors, ui, false, |i: usize, j: usize| {
                        // *state.envelope_location.lock().unwrap() =
                        //     EnvelopeLocation::MasterEffector(i, j);
                    });
                }
            }
        });

        drop(synth);
        ui.collapsing("Wavetable lab", |ui| {
            state.wavetable_lab.lock().unwrap().show(
                ui,
                Some(|tree| {
                    state
                        .synth
                        .lock()
                        .unwrap()
                        .voice
                        .wavetable_settings
                        .set_custom_wavetable(tree);
                }),
            );
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

fn wavetable_seed(synth: &mut std::sync::MutexGuard<crate::synth::MySynth>, ui: &mut egui::Ui) {
    if synth.voice.wavetable_settings.is_custom_wavetable() {
        ui.horizontal(|ui| {
            ui.label("Custom");
            if ui.button("reset").clicked() {
                synth.voice.wavetable_settings.clear_custom_wavetable();
            }
        });
        return;
    }
    let mut seed = synth.voice.wavetable_settings.seed();
    if ui.add(egui::widgets::DragValue::new(&mut seed)).changed() {
        synth.voice.wavetable_settings.set_seed(seed);
    };
}

fn envelope(ui: &mut egui::Ui, envelope: &mut corus_v2::nodes::envelope::Envelope) {
    let release_time: f64 = envelope.points.iter().map(|p| p.0).sum();
    egui::Frame::canvas(ui.style()).show(ui, |ui| {
        let (_id, rect) = ui.allocate_space(egui::vec2(80.0, 40.0));
        let to_screen =
            emath::RectTransform::from_to(egui::Rect::from_x_y_ranges(0.0..=1.0, -1.0..=0.0), rect);
        let w = rect.width() as usize;
        let mut points = vec![];
        for i in 0..=w {
            let p = i as f64 / w as f64 * (release_time + envelope.release_length);
            let v = if p < release_time {
                envelope.compute_level(p)
            } else {
                envelope.compute_release(envelope.points.last().unwrap().1, p - release_time)
            };
            points.push(to_screen * egui::pos2(i as f32 / w as f32, -v as f32));
        }
        ui.painter().add(egui::Shape::line(
            points,
            egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
        ));
    });

    fn env_curve_know(ui: &mut egui::Ui, curve: &mut corus_v2::nodes::envelope::Curve) {
        let mut l = curve.to_level();
        if ui
            .add(crate::widgets::knob::knob(-3.0..3.0, &mut l))
            .changed()
        {
            *curve = corus_v2::nodes::envelope::Curve::from_level(l);
        };
    }

    ui.horizontal(|ui| {
        ui.add(crate::widgets::knob::knob_named(
            0.0..1.0,
            &mut envelope.points[0].0,
            "attack",
        ));
        env_curve_know(ui, &mut envelope.points[0].2);
        ui.add(crate::widgets::knob::knob_named(
            0.0..8.0,
            &mut envelope.points[1].0,
            "decay",
        ));
        ui.add(crate::widgets::knob::knob_named(
            0.0..1.0,
            &mut envelope.points[1].1,
            "sustain",
        ));
        env_curve_know(ui, &mut envelope.points[1].2);
        ui.add(crate::widgets::knob::knob_named(
            0.0..1.0,
            &mut envelope.release_length,
            "release",
        ));
        env_curve_know(ui, &mut envelope.release_curve);
    });
}

fn lfo(ui: &mut egui::Ui, lfo: &mut crate::synth::param_f64::Lfo) {
    ui.horizontal(|ui| {
        ui.add(crate::widgets::knob::knob_named(
            0.001..100.0,
            &mut lfo.frequency,
            "freq",
        ));
        ui.add(crate::widgets::knob::knob_named(
            -10.0..10.0,
            &mut lfo.amp,
            "amp",
        ));
    });
}

fn effectors(
    effectors: &mut Vec<(bool, crate::synth::effectors::Effector)>,
    ui: &mut egui::Ui,
    is_voice: bool,
    setter: impl Fn(usize, usize),
) {
    for (i, (enabled, effector)) in effectors.iter_mut().enumerate() {
        ui.horizontal(|ui| {
            ui.add(egui::widgets::Checkbox::new(enabled, effector.name()));
            use crate::synth::effectors::Effector;

            match effector {
                Effector::Filter { frequency, q } => {
                    add_knob(ui, frequency, 20.0..10000.0, is_voice, || setter(i, 0));
                    add_knob(ui, q, 0.7..10.0, is_voice, || setter(i, 1));
                }
                Effector::Phaser => {}
                Effector::Chorus => {}
                Effector::Delay => {}
                Effector::Reverb => {}
                Effector::Gain { gain } => {
                    // ui.add(egui::widgets::Slider::new(gain, 0.0..=1.5));
                    add_knob(ui, gain, 0.0..1.5, is_voice, || setter(i, 0));
                }
                Effector::Compressor {
                    threshold,
                    ratio,
                    attack,
                    release,
                    gain,
                } => {
                    add_knob(ui, threshold, 0.0..1.0, is_voice, || setter(i, 0));
                    add_knob(ui, ratio, 0.0..1.0, is_voice, || setter(i, 1));
                    add_knob(ui, attack, 0.001..1.0, is_voice, || setter(i, 2));
                    add_knob(ui, release, 0.001..1.0, is_voice, || setter(i, 3));
                    add_knob(ui, gain, 0.0..1.5, is_voice, || setter(i, 4));
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
    is_voice: bool,
    on_click: impl Fn() -> (),
) {
    let res = ui
        .add(crate::widgets::knob::knob(range.clone(), &mut param.value))
        .context_menu(|ui| {
            if ui.button("reset").clicked() {
                param.value = 0.0;
            }

            ui.label("global");
            let producer_id = ProducerId::new(0);
            if param
                .consumer
                .producers
                .iter_mut()
                .find(|p| p.1 == producer_id)
                .is_some()
            {
                if ui.button("Remove").clicked() {
                    param.consumer.producers.retain(|p| p.1 != producer_id);
                }
                param
                    .consumer
                    .producers
                    .iter_mut()
                    .find(|p| p.1 == producer_id)
                    .map(|p| {
                        ui.add(crate::widgets::knob::knob(range.clone(), &mut p.0));
                    });
            } else {
                if ui.button("Add").clicked() {
                    param.consumer.producers.push((0.0, producer_id));
                }
            }

            if !is_voice {
                return;
            }
            ui.label("voice");
            for i in 0..2 {
                let producer_id = ProducerId::new(i);
                if param
                    .voice_consumer
                    .producers
                    .iter_mut()
                    .find(|p| p.1 == producer_id)
                    .is_some()
                {
                    if ui.button("Remove").clicked() {
                        param
                            .voice_consumer
                            .producers
                            .retain(|p| p.1 != producer_id);
                    }
                    param
                        .voice_consumer
                        .producers
                        .iter_mut()
                        .find(|p| p.1 == producer_id)
                        .map(|p| {
                            ui.add(crate::widgets::knob::knob(range.clone(), &mut p.0));
                        });
                } else {
                    if ui.button("Add").clicked() {
                        param.voice_consumer.producers.push((0.0, producer_id));
                    }
                }
            }
        });
    if res.clicked() {
        on_click();
    }
}
