use super::{Node, ProcContext};

pub struct Amp<T, A, B>
where
    T: Clone + 'static + std::ops::Mul<Output = T>,
    A: Node<T>,
    B: Node<T>,
{
    input: A,
    gain: B,
    _t: std::marker::PhantomData<T>,
}

impl<T, A, B> Amp<T, A, B>
where
    T: Clone + 'static + std::ops::Mul<Output = T>,
    A: Node<T>,
    B: Node<T>,
{
    pub fn new(input: A, gain: B) -> Self {
        Amp {
            input,
            gain,
            _t: Default::default(),
        }
    }
}

impl<T, A, B> Node<T> for Amp<T, A, B>
where
    T: Clone + 'static + std::ops::Mul<Output = T>,
    A: Node<T>,
    B: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.input.proc(ctx) * self.gain.proc(ctx)
    }

    fn lock(&mut self) {
        self.input.lock();
        self.gain.lock();
    }

    fn unlock(&mut self) {
        self.input.unlock();
        self.gain.unlock();
    }
}
