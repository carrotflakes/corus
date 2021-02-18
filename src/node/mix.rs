use std::iter::Sum;

use super::{Node, ProcContext};

pub struct Mix<T, DA>
where
    T: Clone + 'static + Sum,
    DA: AsMut<dyn Node<T>>,
{
    nodes: Vec<DA>,
    _t: std::marker::PhantomData<T>,
}

impl<T, DA> Mix<T, DA>
where
    T: Clone + 'static + Sum,
    DA: AsMut<dyn Node<T>>,
{
    pub fn new(nodes: Vec<DA>) -> Self {
        Mix {
            nodes,
            _t: Default::default(),
        }
    }
}

impl<T, DA> Node<T> for Mix<T, DA>
where
    T: Clone + 'static + Sum,
    DA: AsMut<dyn Node<T>>,
{
    fn proc(&mut self, ctx: &ProcContext) -> T {
        self.nodes.iter_mut().map(|n| n.as_mut().proc(ctx)).sum()
    }

    fn lock(&mut self) {
        for node in &mut self.nodes {
            node.as_mut().lock();
        }
    }

    fn unlock(&mut self) {
        for node in &mut self.nodes {
            node.as_mut().unlock();
        }
    }
}

impl<T, DA> AsMut<Self> for Mix<T, DA>
where
    T: Clone + 'static + Sum,
    DA: AsMut<dyn Node<T>>,
{
    fn as_mut(&mut self) -> &mut Mix<T, DA> {
        self
    }
}
