use nih_plug_egui::egui::{self, CursorIcon, Label};

pub fn knob_ui(
    ui: &mut egui::Ui,
    range: std::ops::Range<f64>,
    value: &mut f64,
    name: Option<&str>,
    printer: impl Fn(f64) -> String,
) -> egui::Response {
    let desired_size = egui::vec2(ui.spacing().interact_size.y, ui.spacing().interact_size.y);

    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::drag());

    response = response.on_hover_cursor(CursorIcon::ResizeVertical);

    let mut show_tip = false;

    if response.dragged() {
        let delta = -response.drag_delta().y as f64;
        *value = (*value + delta * (range.end - range.start) / 100.0).clamp(range.start, range.end);
        response.mark_changed();
        ui.output().cursor_icon = CursorIcon::ResizeVertical;
        show_tip = true;
    }
    if response.hovered() {
        show_tip = true;
    }

    response.widget_info(|| egui::WidgetInfo::drag_value(*value as f64));

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact_selectable(&response, true);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter()
            .rect(rect, radius, visuals.bg_fill, visuals.bg_stroke);
        let center = rect.center();
        let a = (*value - range.start) / (range.end - range.start);
        let a = std::f64::consts::TAU * (0.6 - a * 0.7);
        let a = a as f32;
        let v = egui::vec2(a.cos(), -a.sin()) * (radius * 0.75);
        ui.painter()
            .line_segment([center, center + v], visuals.fg_stroke);

        if show_tip {
            egui::containers::show_tooltip_for(
                &response.ctx,
                response.id.with("__tooltip"),
                &response.rect,
                |ui| {
                    ui.add(Label::new(if let Some(name) = name {
                        format!("{} {}", name, printer(*value))
                    } else {
                        format!("{}", printer(*value))
                    }))
                },
            );
        }
    }

    response
}

pub fn knob<'a>(
    range: std::ops::Range<f64>,
    value: &'a mut f64,
    name: &'a str,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| knob_ui(ui, range, value, Some(name), |v| format!("{:.2}", v))
}

pub fn knob_log<'a>(
    range: std::ops::Range<f64>,
    value: &'a mut f64,
    name: &'a str,
) -> impl egui::Widget + 'a {
    move |ui: &mut egui::Ui| {
        let mut v = value.log10();
        let range = range.start.log10()..range.end.log10();
        let res = knob_ui(ui, range, &mut v, Some(name), |v| {
            format!("{:.2}", 10.0f64.powf(v))
        });
        *value = 10.0f64.powf(v);
        res
    }
}
