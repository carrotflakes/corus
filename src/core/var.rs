use crate::EventListener;

use super::{Node, ProcContext};

#[derive(Clone)]
pub struct Var<T: Clone + 'static> {
    value: T,
}

impl<T: Clone + 'static> Var<T> {
    pub fn new(value: T) -> Self {
        Var { value }
    }

    pub fn from<S: Clone + 'static + Into<T>>(src: S) -> Self {
        Var { value: src.into() }
    }
}

impl<T: Clone + 'static> Node for Var<T> {
    type Output = T;

    #[inline]
    fn proc(&mut self, _ctx: &ProcContext) -> T {
        self.value.clone()
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
}

pub enum VarEvent<T: Clone + 'static> {
    SetValue(T),
}

impl<T: Clone + 'static> EventListener<VarEvent<T>> for Var<T> {
    #[inline]
    fn apply_event(&mut self, _time: f64, event: &VarEvent<T>) {
        match event {
            VarEvent::SetValue(value) => {
                self.value = value.clone();
            }
        }
    }
}
