use nih_plug::prelude::*;
use nih_plug_egui::egui;
use std::sync::{Arc, Mutex};

use crate::Synth;

pub fn editor_ui(
    egui_ctx: &egui::Context,
    _setter: &ParamSetter<'_>,
    state: &mut Arc<Mutex<Synth>>,
) {
    egui::CentralPanel::default().show(egui_ctx, |ui| {
        let mut synth = state.lock().unwrap();
        ui.horizontal(|ui| {
            if ui
                .add(egui::widgets::DragValue::new(&mut synth.sound_speed).clamp_range(1.0..=6.0))
                .changed()
            {
                synth.benihora = None;
            }
            ui.label("sound speed");
            if ui
                .add(egui::widgets::DragValue::new(&mut synth.seed).clamp_range(0..=100))
                .changed()
            {
                synth.benihora = None;
            }
            ui.label("seed");
        });
        if synth.benihora.is_some() {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.add(egui::Slider::new(
                        &mut synth.benihora_params.frequency_pid.kp,
                        0.0..=1000.0,
                    ));
                    ui.add(egui::Slider::new(
                        &mut synth.benihora_params.frequency_pid.ki,
                        0.0..=1000.0,
                    ));
                    ui.add(egui::Slider::new(
                        &mut synth.benihora_params.frequency_pid.kd,
                        -0.9..=0.9,
                    ));
                    ui.add(egui::Slider::new(
                        &mut synth.benihora_params.intensity_pid.kp,
                        0.0..=1000.0,
                    ));
                    ui.add(egui::Slider::new(
                        &mut synth.benihora_params.intensity_pid.ki,
                        0.0..=1000.0,
                    ));
                    ui.add(egui::Slider::new(
                        &mut synth.benihora_params.intensity_pid.kd,
                        -0.9..=0.9,
                    ));
                });

                egui::Frame::canvas(ui.style()).show(ui, |ui| {
                    let (_id, rect) = ui.allocate_space(egui::vec2(100.0, 100.0));
                    let to_screen = egui::emath::RectTransform::from_to(
                        egui::Rect::from_x_y_ranges(0.0..=1.0, -1.1..=0.2),
                        rect,
                    );

                    let stroke = egui::Stroke::new(1.0, egui::Color32::DARK_GRAY);
                    ui.painter().line_segment(
                        [
                            to_screen * egui::pos2(0.0, 0.0),
                            to_screen * egui::pos2(1.0, 0.0),
                        ],
                        stroke,
                    );
                    ui.painter().line_segment(
                        [
                            to_screen * egui::pos2(0.0, -1.0),
                            to_screen * egui::pos2(1.0, -1.0),
                        ],
                        stroke,
                    );

                    for (j, ty) in [1, 0, 0, 0].iter().enumerate() {
                        let history = &synth.benihora.as_ref().unwrap().history;
                        let mut points = vec![];
                        let w = rect.width() as usize;
                        for i in 0..=w {
                            if i >= history.len() {
                                break;
                            }
                            let p: f64 = i as f64 / w as f64;
                            let v = history[history.len() - i - 1][j] as f32;
                            let v = match *ty {
                                0 => v,
                                1 => ((v / 440.0).log2() + 2.0) / 5.0,
                                _ => unreachable!(),
                            };
                            points.push(to_screen * egui::pos2(p as f32, -v));
                        }

                        let color = [
                            egui::Color32::from_rgb(0xff, 0x00, 0x00),
                            egui::Color32::from_rgb(0xff, 0x88, 0x00),
                            egui::Color32::from_rgb(0x88, 0xff, 0x00),
                            egui::Color32::from_rgb(0x00, 0xff, 0x00),
                        ][j];
                        ui.painter()
                            .add(egui::Shape::line(points, egui::Stroke::new(1.0, color)));
                    }
                });
            });
        }
    });
}
