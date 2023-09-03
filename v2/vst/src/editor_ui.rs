use std::{cell::RefCell, sync::Arc};

use crate::{
    synth::{
        bender::Bender,
        effectors::{FilterType, ShaperType},
        param_pool::ProducerId,
    },
    MyPluginParams,
};
use nih_plug::prelude::*;
use nih_plug_egui::egui::{self, emath};
use rustfft::num_complex::Complex64;

thread_local! {
    pub static FFT_PLANNER: RefCell<rustfft::FftPlanner<f64>> = RefCell::new(rustfft::FftPlanner::new());
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

        ui.collapsing("Generator", |ui| {
            for (i, osc) in synth.voice.oscs.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    generator_ui(ui, osc);
                });
            }
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

            ui.horizontal(|ui| {
                for l in synth.voice.lfos.iter_mut() {
                    lfo(ui, l);
                }

                for l in synth.lfos.iter_mut() {
                    lfo(ui, l);
                }
            });
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
                    state.synth.lock().unwrap().voice.oscs[0]
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

fn generator_ui(ui: &mut egui::Ui, osc: &mut crate::synth::Osc) {
    ui.horizontal(|ui| {
        let wt = { osc.wavetable_settings.wavetable() };
        let main_wt_id = ui.id().with("main_wt");
        let res = egui::Frame::canvas(ui.style()).show(ui, |ui| {
            let (_id, rect) = ui.allocate_space(egui::vec2(80.0, 80.0));
            let to_screen = emath::RectTransform::from_to(
                egui::Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                rect,
            );
            let mut shapes = vec![];
            let mut points = vec![];

            let fft = ui.data_mut(|d| d.get_persisted::<bool>(main_wt_id).unwrap_or_default());
            if !fft {
                let w = rect.width() as usize;
                for i in 0..=w {
                    let p = i as f64 / w as f64;
                    let v = wt(p % 1.0) as f32;
                    points.push(to_screen * egui::pos2(p as f32, -v));
                }

                shapes.push(egui::Shape::line(
                    points,
                    egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                ));
            } else {
                let res = 256;
                let mut b = (0..res)
                    .map(|i| wt(i as f64 / res as f64))
                    .map(Complex64::from)
                    .collect::<Vec<_>>();
                FFT_PLANNER.with(|planner| {
                    let mut planner = planner.borrow_mut();
                    let fft = planner.plan_fft_forward(res);
                    fft.process(&mut b);
                });
                points.push(to_screen * egui::pos2(0.0, 1.0));
                for i in 0..res / 2 {
                    let x = i as f64 / (res / 2) as f64;
                    let v = (b[i].norm() as f32 / res as f32).sqrt() * 2.0 - 1.0;
                    points.push(to_screen * egui::pos2(x as f32, -v));
                }

                shapes.push(egui::Shape::line(
                    points,
                    egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                ));
            }

            ui.painter().extend(shapes);
        });

        if ui
            .allocate_rect(res.response.rect, egui::Sense::click())
            .clicked()
        {
            ui.data_mut(|d| *d.get_persisted_mut_or_default::<bool>(main_wt_id) ^= true);
        }

        ui.vertical(|ui| {
            wavetable_seed(osc, ui);

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
                        let v = osc.bender.process(osc.bend_level.value, p) as f32;
                        points.push(to_screen * egui::pos2(p as f32, -v));
                    }
                    ui.painter().add(egui::Shape::line(
                        points,
                        egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
                    ));
                });

                let r = osc.bender.level_range();
                add_knob(ui, &mut osc.bend_level, r, true, || ());

                egui::ComboBox::from_label("bend")
                    .selected_text(format!("{:?}", &osc.bender))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut osc.bender, Bender::None, "none");
                        ui.selectable_value(&mut osc.bender, Bender::Quadratic, "quadratic");
                        ui.selectable_value(&mut osc.bender, Bender::Cubic, "cubic");
                        ui.selectable_value(&mut osc.bender, Bender::Sin, "sin");
                        ui.selectable_value(&mut osc.bender, Bender::Cos, "cos");
                    });
            });

            ui.checkbox(&mut osc.wavetable_settings.use_buffer, "prerender");
        });
    });

    ui.horizontal(|ui| {
        add_knob(ui, &mut osc.level, 0.001..1.0, true, || ());
        add_knob(ui, &mut osc.detune, -1.0..1.0, true, || ());
    });

    ui.collapsing("Unison", |ui| {
        ui.horizontal(|ui| {
            ui.add(egui::widgets::DragValue::new(&mut osc.unison_settings.num).clamp_range(1..=10));
            ui.label("voices");
            ui.checkbox(&mut osc.unison_settings.phase_reset, "phase reset");

            ui.separator();

            ui.add(crate::widgets::knob::knob_log(
                0.001..1.0,
                &mut osc.unison_settings.detune,
                "detune",
            ));
            ui.add(crate::widgets::knob::knob_named(
                0.0..1.0,
                &mut osc.unison_settings.stereo_width,
                "stereo width",
            ));
        });
    });
}

fn wavetable_seed(osc: &mut crate::synth::Osc, ui: &mut egui::Ui) {
    if osc.wavetable_settings.is_custom_wavetable() {
        ui.horizontal(|ui| {
            ui.label("Custom");
            if ui.button("reset").clicked() {
                osc.wavetable_settings.clear_custom_wavetable();
            }
        });
        return;
    }
    let mut seed = osc.wavetable_settings.seed();
    if ui.add(egui::widgets::DragValue::new(&mut seed)).changed() {
        osc.wavetable_settings.set_seed(seed);
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
        ui.add(crate::widgets::knob::knob_log(
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
        ui.push_id(("fx", i), |ui| {
            ui.horizontal(|ui| {
                ui.add(egui::widgets::Checkbox::new(enabled, effector.name()));
                use crate::synth::effectors::Effector;

                match effector {
                    Effector::Filter {
                        filter_type,
                        frequency,
                        q,
                        gain,
                    } => {
                        egui::ComboBox::from_label("type")
                            .selected_text(filter_type.name())
                            .show_ui(ui, |ui| {
                                for ft in &FilterType::ALL {
                                    ui.selectable_value(filter_type, *ft, ft.name());
                                }
                            });
                        add_knob(ui, frequency, 20.0..10000.0, is_voice, || setter(i, 0));
                        add_knob(ui, q, 0.7..10.0, is_voice, || setter(i, 1));
                        add_knob(ui, gain, -20.0..20.0, is_voice, || setter(i, 2));
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
                    Effector::Shaper { pre_gain, r#type } => {
                        add_knob(ui, pre_gain, 0.0..32.0, is_voice, || setter(i, 0));
                        egui::ComboBox::from_label("type")
                            .selected_text(r#type.name())
                            .show_ui(ui, |ui| {
                                for st in &ShaperType::ALL {
                                    ui.selectable_value(r#type, *st, st.name());
                                }
                            });
                    }
                }
            });
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
            for i in 0..2 {
                let producer_id = ProducerId::new(i);
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
            }

            if !is_voice {
                return;
            }
            ui.label("voice");
            for i in 0..4 {
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
