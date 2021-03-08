use std::any::Any;

use crate::{context::Context, interface::*};

use super::{Component, text_box::TextBox, wire::{Wire, WireDown}};


pub struct TextBoxContainer<U: Ui> {
    pub text_boxes: Vec<TextBox<U>>,
    pub wire: Option<Wire>,
}

impl<U: Ui> TextBoxContainer<U> {
    pub fn new(text_boxes: Vec<TextBox<U>>) -> Self { Self { text_boxes, wire: None } }

    pub fn get_text_box_by_id_mut(&mut self, id: usize) -> Option<&mut TextBox<U>> {
        self.text_boxes.iter_mut().find(|tb| tb.id == id)
    }
}

impl<U: Ui> Component<U> for TextBoxContainer<U> {
    fn update(&mut self) {
        for text_box in &mut self.text_boxes {
            text_box.update();
        }
        // for text_box in &mut self.text_boxes {
        // }
    }

    fn message_receive(&mut self, ctx: &mut Context<U>, message: &Box<dyn Any>) -> bool {
        for text_box in &mut self.text_boxes {
            if text_box.message_receive(ctx, message) {
                return true;
            }
        }
        if let Some(wire) = message.downcast_ref::<Wire>() {
            self.wire = Some(wire.clone());
        }
        if let Some(_) = message.downcast_ref::<WireDown>() {
            if let Some(wire) = &self.wire {
                let mut a = None;
                for tb in &self.text_boxes {
                    for or in tb.output_rects() {
                        if or.contains_point(wire.end) {
                            a = Some(tb.id);
                        }
                    }
                }
                if let Some(id) = a {
                    if let Some(tb1) = self.get_text_box_by_id_mut(wire.text_box_id) {
                        tb1.inputs.push(id);
                    }
                }
            }
        }
        if let Some(wire) = &mut self.wire {
            wire.message_receive(ctx, message);
        }
        false
    }

    fn draw(&mut self, ctx: &mut Context<U>) {
        for text_box in &mut self.text_boxes {
            text_box.draw(ctx);
        }
        for text_box in &self.text_boxes {
            for i in &text_box.inputs {
                let other_text_box = self.text_boxes.iter().find(|tb| tb.id == *i).unwrap();
                let p1 = Point::new(text_box.rect.0, text_box.rect.1);
                let p2 = Point::new(other_text_box.rect.0, other_text_box.rect.1);
                ctx.canvas.draw_line(p1, p2).unwrap();
            }
        }
        if let Some(wire) = &mut self.wire {
            wire.draw(ctx);
        }
    }
}
