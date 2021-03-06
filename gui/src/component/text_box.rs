use std::{any::Any, marker::PhantomData};

use crate::{context::Context, interface::*};

use super::{Component, wire::Wire};

pub struct TextBox<U: Ui> {
    pub id: usize,
    pub str: String,
    pub rect: Rect,
    pub inputs: Vec<usize>,
    touch: Option<TextBoxState>,
    _t: PhantomData<dyn Fn() -> U>,
}

impl<U: Ui> TextBox<U> {
    pub fn new(id: usize, str: String, rect: Rect) -> Self {
        Self {
            id,
            str,
            rect,
            inputs: vec![],
            touch: None,
            _t: Default::default(),
        }
    }

    pub fn input_rects(&self) -> impl Iterator<Item = Rect> {
        let rect = self.rect;
        (0..1).map(move |i| {
            Rect::new(rect.0 + i * 20, rect.1 - 5, 10, 5)
        })
    }

    pub fn output_rects(&self) -> impl Iterator<Item = Rect> {
        let rect = self.rect;
        (0..1).map(move |i| {
            Rect::new(rect.0 + i * 20, rect.1 + rect.3 as i32, 10, 5)
        })
    }
}

impl<U: Ui> Component<U> for TextBox<U> {
    fn update(&mut self) {}

    fn message_receive(&mut self, ctx: &mut Context<U>, message: &Box<dyn Any>) -> bool {
        if let Some(event) = message.downcast_ref::<Event>() {
            match event {
                Event::MouseMotion {
                    timestamp,
                    window_id,
                    which,
                    x,
                    y,
                    xrel,
                    yrel,
                } => {
                    if let Some(TextBoxState::MouseDrag) = self.touch {
                        self.rect.0 = self.rect.0 + xrel;
                        self.rect.1 = self.rect.1 + yrel;
                        return true;
                    }
                }
                Event::MouseButtonDown {
                    timestamp,
                    window_id,
                    which,
                    mouse_btn,
                    clicks,
                    x,
                    y,
                } => {
                    if *mouse_btn == MouseButton::Left {
                        for ir in self.input_rects() {
                            if ir.contains_point(Point::new(*x, *y)) {
                                ctx.messages.push(Box::new(Wire::new(
                                    self.id,
                                    Point::new(ir.0, ir.1),
                                    Point::new(ir.0, ir.1),
                                )));
                            }
                        }
                        if self.rect.contains_point(Point::new(*x, *y)) {
                            self.touch = Some(TextBoxState::MouseDrag);
                            return true;
                        }
                    }
                }
                Event::MouseButtonUp {
                    timestamp,
                    window_id,
                    which,
                    mouse_btn,
                    clicks,
                    x,
                    y,
                } => {
                    if *mouse_btn == MouseButton::Left {
                        if *clicks == 2 && self.rect.contains_point(Point::new(*x, *y)) {
                            ctx.text_input_start(self.str.clone(), self.rect.clone(), Some(self));
                            self.str = String::new();
                            self.touch = None;
                            return true;
                        }
                        if let Some(TextBoxState::MouseDrag) = self.touch {
                            self.touch = None;
                            return true;
                        }
                    }
                }
                Event::MouseWheel {
                    timestamp,
                    window_id,
                    which,
                    x,
                    y,
                } => {}
                Event::FingerDown {
                    timestamp,
                    touch_id,
                    finger_id,
                    x,
                    y,
                    dx,
                    dy,
                    pressure,
                } => {
                    if self.rect.contains_point(Point::new(*x as i32, *y as i32)) {
                        self.touch = Some(TextBoxState::FingerDrag(*finger_id));
                        return true;
                    }
                }
                Event::FingerUp { finger_id, .. } => {
                    if let Some(TextBoxState::FingerDrag(finger_id)) = self.touch {
                        self.touch = None;
                        return true;
                    }
                }
                Event::FingerMotion {
                    finger_id, dx, dy, ..
                } => {
                    if let Some(TextBoxState::MouseDrag) = self.touch {
                        self.rect.0 = self.rect.0 + *dx as i32;
                        self.rect.1 = self.rect.1 + *dy as i32;
                        return true;
                    }
                }
                _ => {}
            }
        } else if let Some((str, component_ptr)) = message.downcast_ref::<(String, Option<*const dyn Component<U>>)>() {
            if component_ptr.clone() == Some(self) {
                self.str = str.clone();
                return true;
            }
        }
        false
    }

    fn draw(&mut self, ctx: &mut Context<U>) {
        ctx.canvas.set_draw_color(RGB(0, 0, 0));
        for input_rect in self.input_rects() {
            ctx.canvas.draw_rect(input_rect);
        }
        for output_rect in self.output_rects() {
            ctx.canvas.draw_rect(output_rect);
        }
        ctx.canvas.draw_rect(self.rect.clone());
        ctx.draw_text(&self.str, self.rect.0, self.rect.1);
    }
}

enum TextBoxState {
    MouseDrag,
    FingerDrag(i64),
}
