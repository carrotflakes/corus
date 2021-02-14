use super::{Node, ProcContext};

pub struct Amp<T: Clone + 'static + std::ops::Mul<Output=T>> {
    input: Box<dyn Node<T>>,
    gain: Box<dyn Node<T>>,
}

impl<T: Clone + 'static + std::ops::Mul<Output=T>> Amp<T> {
    pub fn new(input: Box<dyn Node<T>>, gain: Box<dyn Node<T>>) -> Self {
        Amp {
            input,
            gain
        }
    }
}

impl<T: Clone + 'static + std::ops::Mul<Output=T>> Node<T> for Amp<T> {
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.input.proc(ctx) * self.gain.proc(ctx)
    }
}
