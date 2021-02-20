use crate::node::Node;
use crate::proc_context::ProcContext;

pub struct SampleIterator<T, A, DA>
where
    T: 'static,
    A: Node<T>,
    DA: AsMut<A>,
{
    context: ProcContext,
    node: DA,
    _t: std::marker::PhantomData<T>,
    _a: std::marker::PhantomData<A>,
}

impl<T, A, DA> SampleIterator<T, A, DA>
where
    T: 'static,
    A: Node<T>,
    DA: AsMut<A>,
{
    pub fn new(context: ProcContext, node: DA) -> Self {
        SampleIterator {
            context,
            node,
            _t: Default::default(),
            _a: Default::default(),
        }
    }
}

impl<T, A, DA> Iterator for SampleIterator<T, A, DA>
where
    T: 'static,
    A: Node<T>,
    DA: AsMut<A>,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.context.sample(&mut self.node))
    }
}
