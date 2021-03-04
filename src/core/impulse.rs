use crate::Event;

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

impl<T> Node<T> for Impulse<T>
where
    T: Clone + 'static + Default,
{
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

impl<T: Clone + 'static + Default> Event for ImpulseEvent<T> {
    type Target = Impulse<T>;

    fn dispatch(&self, _time: f64, node: &mut Self::Target) {
        match self {
            ImpulseEvent::Trigger => {
                node.fired = false;
            }
            ImpulseEvent::SetValue(value) => {
                node.value = value.clone();
            }
        }
    }
}
