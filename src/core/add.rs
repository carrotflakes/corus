use super::{Node, ProcContext};

pub struct Add<T, A, B>
where
    T: Clone + 'static + std::ops::Add<Output = T>,
    A: Node<T>,
    B: Node<T>,
{
    a: A,
    b: B,
    _t: std::marker::PhantomData<T>,
}

impl<T, A, B> Add<T, A, B>
where
    T: Clone + 'static + std::ops::Add<Output = T>,
    A: Node<T>,
    B: Node<T>,
{
    pub fn new(a: A, b: B) -> Self {
        Add {
            a,
            b,
            _t: Default::default(),
        }
    }
}

impl<T, A, B> Node<T> for Add<T, A, B>
where
    T: Clone + 'static + std::ops::Add<Output = T>,
    A: Node<T>,
    B: Node<T>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.a.proc(ctx) + self.b.proc(ctx)
    }

    fn lock(&mut self) {
        self.a.lock();
        self.b.lock();
    }

    fn unlock(&mut self) {
        self.a.unlock();
        self.b.unlock();
    }
}
