use std::any::Any;

use crate::{context::Context, interface::*};

use super::Component;


pub struct TextInputState<U: Ui> {
    pub inputing: bool,
    pub text: String,
    pub editing_text: String,
    pub cb_end: Option<Box<dyn Fn(&str)>>,
    pub component_ptr: Option<*const dyn Component<U>>,
    pub rect: Rect,
    pub blink_count: usize,
}

impl<U: Ui> TextInputState<U> {
    pub fn new(inputing: bool, text: String) -> Self {
        Self {
            inputing,
            text,
            editing_text: String::new(),
            cb_end: None,
            component_ptr: None,
            rect: Rect::new(0, 0, 1, 1),
            blink_count: 0,
        }
    }

    pub fn event_receive(&mut self, ctx: &mut Context<U>, event: &Event) -> bool {
        if !self.inputing {
            return false;
        }
        match event {
            Event::TextEditing {
                timestamp,
                window_id,
                text,
                start,
                length,
            } => {
                self.editing_text = text.clone();
                true
            }
            Event::TextInput {
                timestamp,
                window_id,
                text,
            } => {
                self.text.push_str(text);
                true
            }
            Event::KeyDown {
                keycode: Some(key),
                ..
            } => {
                match key {
                    Keycode::Backspace => {
                        self.text.pop();
                        true
                    }
                    Keycode::Return => {
                        self.inputing = false;
                        let text = self.text.clone();
                        self.text = String::new();
                        if let Some(cb) = &self.cb_end {
                            cb(&self.text);
                        }
                        ctx.messages.push(Box::new((text, self.component_ptr) as (String, Option<*const dyn Component<U>>)));
                        true
                    }
                    Keycode::Escape => {
                        false
                    }
                    Keycode::Delete => {
                        false
                    }
                    _ => {
                        false
                    }
                }
            }
            _ => false,
        }
    }
}

impl<U: Ui> Component<U> for TextInputState<U> {
    fn update(&mut self) {
    }

    fn message_receive(&mut self, ctx: &mut Context<U>, message: &Box<dyn Any>) -> bool {
        if let Some(event) = message.downcast_ref::<Event>() {
            self.event_receive(ctx, event)
        } else {
            false
        }
    }

    fn draw(&mut self, ctx: &mut Context<U>) {
        if self.inputing {
            self.blink_count += 1;
            let size = ctx.draw_text(
                self.text.as_str(),
                self.rect.0,
                self.rect.1,
            );
            if (self.blink_count / 4) % 2 != 0 {
                ctx.canvas.draw_line(
                    Point::new(self.rect.0 + size.0 as i32, self.rect.1),
                    Point::new(self.rect.0 + size.0 as i32, self.rect.1 + self.rect.3 as i32)
                ).unwrap();
            }
        }
    }
}
