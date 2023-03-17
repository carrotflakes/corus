use std::sync::Arc;

use crate::{synth::bender::Bender, MyPluginParams};
use nih_plug::prelude::*;
use nih_plug_egui::egui::{self, emath};

pub fn editor_updator(
    egui_ctx: &egui::Context,
    setter: &ParamSetter,
    state: &mut Arc<MyPluginParams>,
) {
    egui::CentralPanel::default().show(egui_ctx, |ui| {
        let mut synth = state.synth.lock().unwrap();

        ui.horizontal(|ui| {
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

            ui.vertical(|ui| {
                ui.add(egui::widgets::DragValue::new(&mut synth.voice_params.seed));

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
                            egui::Stroke::new(1.0, egui::Color32::RED),
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
                });

                egui::ComboBox::from_label("bend")
                    .selected_text(format!("{:?}", &synth.voice_params.bender))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut synth.voice_params.bender, Bender::None, "none");
                        ui.selectable_value(
                            &mut synth.voice_params.bender,
                            Bender::Quadratic,
                            "quadratic",
                        );
                        ui.selectable_value(&mut synth.voice_params.bender, Bender::Cubic, "cubic");
                        ui.selectable_value(&mut synth.voice_params.bender, Bender::Sin, "sin");
                        ui.selectable_value(&mut synth.voice_params.bender, Bender::Cos, "cos");
                    });
            });
        });
        ui.add(egui::widgets::Checkbox::new(
            &mut synth.voice_params.filter_enabled,
            "Filter",
        ));
        ui.collapsing("Unison", |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::widgets::DragValue::new(&mut synth.unison_num).clamp_range(1..=10));
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

        ui.collapsing("Master", |ui| {
            for (enabled, effector) in &mut synth.effectors {
                ui.horizontal(|ui| {
                    ui.add(egui::widgets::Checkbox::new(enabled, effector.name()));
                    use crate::synth::effectors::Effector;
                    #[allow(unused_variables)]
                    match effector {
                        Effector::Filter { frequency, q } => {
                            ui.add(crate::widgets::knob::knob(20.0..10000.0, frequency));
                            ui.add(crate::widgets::knob::knob(0.7..10.0, q));
                        }
                        Effector::Phaser => {}
                        Effector::Chorus => {}
                        Effector::Delay => {}
                        Effector::Reverb => {}
                        Effector::Gain { gain } => {
                            // ui.add(egui::widgets::Slider::new(gain, 0.0..=1.5));
                            ui.add(crate::widgets::knob::knob(0.0..1.5, gain));
                        }
                        Effector::Tanh {} => {}
                    }
                });
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
