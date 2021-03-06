use std::{
    any::Any,
    sync::{Arc, Mutex},
};

use crate::interface::*;
use crate::{
    component::{text_input_state::TextInputState, Component},
    interface,
};

pub struct Context<U: Ui> {
    pub canvas: <<<U as interface::Ui>::VideoSubsystem as interface::VideoSubsystem>::Window as interface::Window>::Canvas,
    pub font: <<<<U as interface::Ui>::VideoSubsystem as interface::VideoSubsystem>::Window as interface::Window>::Canvas as interface::Canvas>::Font,
    pub text_inputing_state: Arc<Mutex<TextInputState<U>>>,
    pub video_subsys: <U as interface::Ui>::VideoSubsystem,
    pub messages: Vec<Box<dyn Any>>,
}

impl<U: Ui> Context<U> {
    pub fn new(
        canvas: <<<U as interface::Ui>::VideoSubsystem as interface::VideoSubsystem>::Window as interface::Window>::Canvas,
        font: <<<<U as interface::Ui>::VideoSubsystem as interface::VideoSubsystem>::Window as interface::Window>::Canvas as interface::Canvas>::Font,
        text_inputing_state: TextInputState<U>,
        video_subsys: <U as interface::Ui>::VideoSubsystem,
    ) -> Self {
        Self {
            canvas,
            font,
            text_inputing_state: Arc::new(Mutex::new(text_inputing_state)),
            video_subsys,
            messages: Vec::new(),
        }
    }

    pub fn draw_text(&mut self, text: &str, x: i32, y: i32) -> (u32, u32) {
        self.canvas.draw_text(&mut self.font, text, x, y)
    }

    pub fn text_input_start(
        &mut self,
        text: String,
        rect: Rect,
        component_ptr: Option<*const dyn Component<U>>,
    ) {
        self.video_subsys.text_input_start(rect.clone());
        let mut tis = self.text_inputing_state.lock().unwrap();
        tis.inputing = true;
        tis.text = text;
        tis.component_ptr = component_ptr;
        tis.rect = rect;
        tis.blink_count = 0;
    }
}
