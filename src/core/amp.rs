use super::{Node, ProcContext};

pub struct Amp<T, A, B, DA, DB>
where
    T: Clone + 'static + std::ops::Mul<Output = T>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    input: DA,
    gain: DB,
    _t: std::marker::PhantomData<T>,
    _a: std::marker::PhantomData<A>,
    _b: std::marker::PhantomData<B>,
}

impl<T, A, B, DA, DB> Amp<T, A, B, DA, DB>
where
    T: Clone + 'static + std::ops::Mul<Output = T>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    pub fn new(input: DA, gain: DB) -> Self {
        Amp {
            input,
            gain,
            _t: Default::default(),
            _a: Default::default(),
            _b: Default::default(),
        }
    }
}

impl<T, A, B, DA, DB> Node<T> for Amp<T, A, B, DA, DB>
where
    T: Clone + 'static + std::ops::Mul<Output = T>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.input.as_mut().proc(ctx) * self.gain.as_mut().proc(ctx)
    }

    fn lock(&mut self) {
        self.input.as_mut().lock();
        self.gain.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.input.as_mut().unlock();
        self.gain.as_mut().unlock();
    }
}

impl<T, A, B, DA, DB> AsMut<Self> for Amp<T, A, B, DA, DB>
where
    T: Clone + 'static + std::ops::Mul<Output = T>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Amp<T, A, B, DA, DB> {
        self
    }
}
