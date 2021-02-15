use std::iter::Sum;

use super::{Node, ProcContext};

pub struct Add<T, DA>
where
    T: Clone + 'static + Sum,
    DA: AsMut<dyn Node<T>>,
{
    nodes: Vec<DA>,
    _t: std::marker::PhantomData<T>,
}

impl<T, DA> Add<T, DA>
where
    T: Clone + 'static + Sum,
    DA: AsMut<dyn Node<T>>,
{
    pub fn new(nodes: Vec<DA>) -> Self {
        Add {
            nodes,
            _t: Default::default(),
        }
    }
}

impl<T, DA> Node<T> for Add<T, DA>
where
    T: Clone + 'static + Sum,
    DA: AsMut<dyn Node<T>>,
{
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.nodes.iter_mut().map(|n| n.as_mut().proc(ctx)).sum()
    }
}

impl<T, DA> AsMut<Self> for Add<T, DA>
where
    T: Clone + 'static + Sum,
    DA: AsMut<dyn Node<T>>,
{
    fn as_mut(&mut self) -> &mut Add<T, DA> {
        self
    }
}
