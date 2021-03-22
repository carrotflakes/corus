use super::{Node, ProcContext};

pub struct Mul<T, A, B>
where
    T: Clone + 'static + std::ops::Mul<Output = T>,
    A: Node<T>,
    B: Node<T>,
{
    input1: A,
    input2: B,
    _t: std::marker::PhantomData<T>,
}

impl<T, A, B> Mul<T, A, B>
where
    T: Clone + 'static + std::ops::Mul<Output = T>,
    A: Node<T>,
    B: Node<T>,
{
    pub fn new(input1: A, input2: B) -> Self {
        Mul {
            input1,
            input2,
            _t: Default::default(),
        }
    }
}

impl<T, A, B> Node<T> for Mul<T, A, B>
where
    T: Clone + 'static + std::ops::Mul<Output = T>,
    A: Node<T>,
    B: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.input1.proc(ctx) * self.input2.proc(ctx)
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.input1.lock(ctx);
        self.input2.lock(ctx);
    }

    fn unlock(&mut self) {
        self.input1.unlock();
        self.input2.unlock();
    }
}
