use nih_plug_egui::egui::{self, emath};

pub fn wavetable_view(
    ui: &mut egui::Ui,
    size: egui::Vec2,
    f: impl Fn(f64) -> f64,
    selected: bool,
) -> egui::Response {
    let mut stroke = ui.style().visuals.window_stroke();
    if selected {
        stroke.color = egui::Color32::GRAY;
    }

    let res = egui::Frame::canvas(ui.style())
        .stroke(stroke)
        .show(ui, |ui| {
            let (_id, rect) = ui.allocate_space(size);
            let to_screen = emath::RectTransform::from_to(
                egui::Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                rect,
            );
            let mut shapes = vec![];

            let w = rect.width() as usize;
            let mut points = vec![];
            for i in 0..=w {
                let p = i as f64 / w as f64;
                let v = f(p % 1.0) as f32;
                points.push(to_screen * egui::pos2(p as f32, -v));
            }
            shapes.push(egui::Shape::line(
                points,
                egui::Stroke::new(1.0, egui::Color32::LIGHT_BLUE),
            ));
            ui.painter().extend(shapes);
        });
    ui.allocate_rect(res.response.rect, egui::Sense::click())
}
