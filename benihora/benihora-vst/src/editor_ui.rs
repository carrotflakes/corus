use nih_plug::prelude::*;
use nih_plug_egui::egui;
use std::sync::{Arc, Mutex};

use crate::{
    knob::{knob, knob_log},
    Synth,
};

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
            ui.checkbox(&mut synth.benihora_params.always_sound, "always");
        });
        if synth.benihora.is_some() {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.add(knob_log(
                            0.1..1000.0,
                            &mut synth.benihora_params.frequency_pid.kp,
                            "frequency kp",
                        ));
                        ui.add(knob_log(
                            0.1..1000.0,
                            &mut synth.benihora_params.frequency_pid.ki,
                            "frequency ki",
                        ));
                        ui.add(knob(
                            -0.9..0.9,
                            &mut synth.benihora_params.frequency_pid.kd,
                            "frequency kd",
                        ));
                        ui.add(knob(
                            0.0..5.0,
                            &mut synth.benihora_params.wobble_amount,
                            "wobble amount",
                        ));
                        ui.add(knob(
                            0.0..0.1,
                            &mut synth.benihora_params.vibrato_amount,
                            "vibrato amount",
                        ));
                        ui.add(knob_log(
                            0.1..20.0,
                            &mut synth.benihora_params.vibrato_frequency,
                            "vibrato frequency",
                        ));
                    });
                    ui.horizontal(|ui| {
                        ui.add(knob_log(
                            0.1..1000.0,
                            &mut synth.benihora_params.intensity_pid.kp,
                            "intensity kp",
                        ));
                        ui.add(knob_log(
                            0.1..1000.0,
                            &mut synth.benihora_params.intensity_pid.ki,
                            "intensity ki",
                        ));
                        ui.add(knob(
                            -0.9..0.9,
                            &mut synth.benihora_params.intensity_pid.kd,
                            "intensity kd",
                        ));
                    });
                    ui.add(knob(
                        0.0..10.0,
                        &mut synth.benihora_params.aspiration_level,
                        "aspiration level",
                    ));
                });

                let view_id = ui.id().with("view");
                let view_mode = ui
                    .data()
                    .get_persisted::<usize>(view_id)
                    .unwrap_or_default();
                if match view_mode {
                    0 => show_tract(ui, &synth.benihora.as_ref().unwrap().benihora.tract),
                    1 => {
                        let history = &synth.benihora.as_ref().unwrap().history;
                        show_history(ui, history)
                    }
                    2 => show_waveform(
                        ui,
                        synth
                            .benihora
                            .as_ref()
                            .unwrap()
                            .waveform_recorder
                            .get_waveform(),
                    ),
                    _ => unreachable!(),
                }
                .clicked()
                {
                    let data = &mut ui.data();
                    let view = data.get_persisted_mut_or_default::<usize>(view_id);
                    *view = (*view + 1) % 3;
                }
            });
        }
    });
}

fn show_tract(ui: &mut egui::Ui, tract: &benihora::tract::Tract) -> egui::Response {
    let res = egui::Frame::canvas(ui.style()).show(ui, |ui| {
        let (_id, rect) = ui.allocate_space(egui::vec2(100.0, 100.0));
        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_x_y_ranges(0.0..=45.0, 0.0..=10.0),
            rect,
        );

        let dx = tract.source.nose_start as f32;
        let dy = 3.75;
        let mut points = vec![];
        points.push(to_screen * egui::pos2(dx, 4.0));
        for (i, d) in tract.current_diameter.nose.iter().enumerate().skip(1) {
            points.push(to_screen * egui::pos2(dx + i as f32, dy - *d as f32));
        }
        let stroke = egui::Stroke::new(1.0, egui::Color32::GRAY);
        ui.painter().add(egui::Shape::line(points, stroke));
        ui.painter().line_segment(
            [
                to_screen * egui::pos2(dx + tract.current_diameter.nose[0] as f32 * 5.0, dy),
                to_screen * egui::pos2(dx + (tract.current_diameter.nose.len() - 1) as f32, dy),
            ],
            stroke,
        );

        let mut points = vec![];
        for (i, d) in tract.current_diameter.mouth.iter().enumerate() {
            points.push(to_screen * egui::pos2(i as f32, (*d + 4.0) as f32));
        }
        let stroke = egui::Stroke::new(1.0, egui::Color32::GRAY);
        ui.painter().add(egui::Shape::line(points, stroke));
        ui.painter().line_segment(
            [
                to_screen * egui::pos2(0.0, 4.0),
                to_screen * egui::pos2((tract.current_diameter.mouth.len() - 1) as f32, 4.0),
            ],
            stroke,
        );
    });
    ui.allocate_rect(res.response.rect, egui::Sense::click())
}

fn show_history(ui: &mut egui::Ui, history: &Vec<[f32; 4]>) -> egui::Response {
    let res = egui::Frame::canvas(ui.style()).show(ui, |ui| {
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
    ui.allocate_rect(res.response.rect, egui::Sense::click())
}

fn show_waveform(ui: &mut egui::Ui, waveform: &[f32]) -> egui::Response {
    let res = egui::Frame::canvas(ui.style()).show(ui, |ui| {
        let (_id, rect) = ui.allocate_space(egui::vec2(100.0, 100.0));
        let to_screen = egui::emath::RectTransform::from_to(
            egui::Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
            rect,
        );

        let points: Vec<_> = waveform
            .iter()
            .enumerate()
            .map(|(i, v)| to_screen * egui::pos2(i as f32 / waveform.len() as f32, -v))
            .collect();
        let stroke = egui::Stroke::new(1.0, egui::Color32::GRAY);
        ui.painter().add(egui::Shape::line(points, stroke));
    });
    ui.allocate_rect(res.response.rect, egui::Sense::click())
}
