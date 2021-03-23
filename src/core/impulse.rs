use crate::EventListener;

use super::{Node, ProcContext};

pub struct Impulse<T>
where
    T: Clone + 'static + Default,
{
    pub value: T,
    pub fired: bool,
}

impl<T> Impulse<T>
where
    T: Clone + 'static + Default,
{
    pub fn new(value: T, fired: bool) -> Self {
        Impulse {
            value,
            fired,
        }
    }
}

impl<T> Node for Impulse<T>
where
    T: Clone + 'static + Default,
{
    type Output = T;

    #[inline]
    fn proc(&mut self, _ctx: &ProcContext) -> T {
        if self.fired {
            Default::default()
        } else {
            self.fired = true;
            self.value.clone()
        }
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
}

pub enum ImpulseEvent<T: Clone + 'static + Default> {
    Trigger,
    SetValue(T),
}

impl<T: Clone + 'static + Default> EventListener<ImpulseEvent<T>> for Impulse<T> {
    #[inline]
    fn apply_event(&mut self, _time: f64, event: &ImpulseEvent<T>) {
        match event {
            ImpulseEvent::Trigger => {
                self.fired = false;
            }
            ImpulseEvent::SetValue(value) => {
                self.value = value.clone();
            }
        }
    }
}
