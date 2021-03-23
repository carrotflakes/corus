use crate::EventListener;

use super::{Node, ProcContext};

#[derive(Clone)]
pub struct Constant<T: Clone + 'static> {
    value: T,
}

impl<T: Clone + 'static> Constant<T> {
    pub fn new(value: T) -> Self {
        Constant { value }
    }

    pub fn from<S: Clone + 'static + Into<T>>(src: S) -> Self {
        Constant { value: src.into() }
    }
}

impl<T: Clone + 'static> Node for Constant<T> {
    type Output = T;

    #[inline]
    fn proc(&mut self, _ctx: &ProcContext) -> T {
        self.value.clone()
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
}

pub enum ConstantEvent<T: Clone + 'static> {
    SetValue(T),
}

impl<T: Clone + 'static> EventListener<ConstantEvent<T>> for Constant<T> {
    #[inline]
    fn apply_event(&mut self, _time: f64, event: &ConstantEvent<T>) {
        match event {
            ConstantEvent::SetValue(value) => {
                self.value = value.clone();
            }
        }
    }
}
