use crate::interface::*;

use super::Component;

#[derive(Clone)]
pub struct Wire {
    pub text_box_id: usize,
    pub start: Point,
    pub end: Point,
    destroyed: bool,
}

impl Wire {
    pub fn new(text_box_id: usize, start: Point, end: Point) -> Self { Self { text_box_id, start, end, destroyed: false } }
}

impl<U: Ui> Component<U> for Wire {
    fn update(&mut self) {
    }

    fn message_receive(&mut self, ctx: &mut crate::context::Context<U>, message: &Box<dyn std::any::Any>) -> bool {
        if let Some(event) = message.downcast_ref::<Event>() {
            match event {
                Event::MouseMotion { timestamp, window_id, which, x, y, xrel, yrel } => {
                    self.end = Point::new(*x, *y);
                }
                Event::MouseButtonUp { timestamp, window_id, which, mouse_btn, clicks, x, y } => {
                    self.destroyed = true;
                    ctx.messages.push(Box::new(WireDown));
                }
                Event::MouseButtonDown { timestamp, window_id, which, mouse_btn, clicks, x, y } => {
                    self.destroyed = true;
                }
                _ => {}
            }
        }
        false
    }

    fn draw(&mut self, ctx: &mut crate::context::Context<U>) {
        if !self.destroyed {
            ctx.canvas.draw_line(self.start, self.end).unwrap();
        }
    }
}

pub struct WireDown;
