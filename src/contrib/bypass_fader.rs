use crate::node::{proc_once_share::ProcOnceShare, Node};

use super::crossfader::{Crossfader, CrossfaderLevel};

pub fn bypass_fader<F, T, A, B, C, DA, DB, DC>(
    node: ProcOnceShare<T, A, DA>,
    wrapper: &dyn Fn(ProcOnceShare<T, A, DA>) -> DB,
    level: DC,
) -> Crossfader<F, T, ProcOnceShare<T, A, DA>, B, C, ProcOnceShare<T, A, DA>, DB, DC>
where
    F: 'static,
    T: CrossfaderLevel<F>,
    A: Node<T> + 'static,
    B: Node<T> + 'static,
    C: Node<F> + 'static,
    DA: AsMut<A> + 'static,
    DB: AsMut<B> + 'static,
    DC: AsMut<C> + 'static,
{
    Crossfader::new(node.clone(), wrapper(node), level)
}
