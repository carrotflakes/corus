use super::{Node, ProcContext};

pub struct Add<T, A, B, DA, DB>
where
    T: Clone + 'static + std::ops::Add<Output = T>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    a: DA,
    b: DB,
    _t: std::marker::PhantomData<T>,
    _a: std::marker::PhantomData<A>,
    _b: std::marker::PhantomData<B>,
}

impl<T, A, B, DA, DB> Add<T, A, B, DA, DB>
where
    T: Clone + 'static + std::ops::Add<Output = T>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    pub fn new(a: DA, b: DB) -> Self {
        Add {
            a,
            b,
            _t: Default::default(),
            _a: Default::default(),
            _b: Default::default(),
        }
    }
}

impl<T, A, B, DA, DB> Node<T> for Add<T, A, B, DA, DB>
where
    T: Clone + 'static + std::ops::Add<Output = T>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.a.as_mut().proc(ctx) + self.b.as_mut().proc(ctx)
    }
}


impl<T, A, B, DA, DB> AsMut<Self> for Add<T, A, B, DA, DB>
where
    T: Clone + 'static + std::ops::Add<Output = T>,
    A: Node<T> + ?Sized,
    B: Node<T> + ?Sized,
    DA: AsMut<A>,
    DB: AsMut<B>,
{
    fn as_mut(&mut self) -> &mut Add<T, A, B, DA, DB> {
        self
    }
}
