use std::ops::Add;

use super::{Node, ProcContext};

pub struct Mix<T, DA>
where
    T: Clone + 'static + Add<Output = T> + Default,
    DA: AsMut<dyn Node<T>>,
{
    nodes: Vec<DA>,
    _t: std::marker::PhantomData<T>,
}

impl<T, DA> Mix<T, DA>
where
    T: Clone + 'static + Add<Output = T> + Default,
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
    T: Clone + 'static + Add<Output = T> + Default,
    DA: AsMut<dyn Node<T>>,
{
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> T {
        let mut v = Default::default();
        for node in self.nodes.iter_mut() {
            v = v + node.as_mut().proc(ctx);
        }
        v
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
    T: Clone + 'static + Add<Output = T> + Default,
    DA: AsMut<dyn Node<T>>,
{
    #[inline]
    fn as_mut(&mut self) -> &mut Mix<T, DA> {
        self
    }
}
