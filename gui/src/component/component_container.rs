use std::any::Any;

use crate::{context::Context, interface::Ui};

use super::Component;


pub struct ComponentContainer<U: Ui> {
    pub components: Vec<Box<dyn Component<U>>>,
}

impl<U: Ui> ComponentContainer<U> {
    pub fn new(components: Vec<Box<dyn Component<U>>>) -> Self { Self { components } }
}

impl<U: Ui> Component<U> for ComponentContainer<U> {
    fn update(&mut self) {
        for component in &mut self.components {
            component.update();
        }
    }

    fn message_receive(&mut self, ctx: &mut Context<U>, message: &Box<dyn Any>) -> bool {
        for component in &mut self.components {
            if component.message_receive(ctx, message) {
                return true;
            }
        }
        false
    }

    fn draw(&mut self, ctx: &mut Context<U>) {
        for component in &mut self.components {
            component.draw(ctx);
        }
    }
}
