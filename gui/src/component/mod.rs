use std::any::Any;

use crate::{context::Context, interface::*};

pub mod component_container;
pub mod text_box;
pub mod text_box_container;
pub mod text_input_state;
pub mod wire;

pub trait Component<U: Ui> {
    fn update(&mut self);
    fn message_receive(&mut self, ctx: &mut Context<U>, message: &Box<dyn Any>) -> bool;
    fn draw(&mut self, ctx: &mut Context<U>);
}
