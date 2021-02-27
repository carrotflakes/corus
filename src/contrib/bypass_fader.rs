use crate::core::{proc_once_share::ProcOnceShare, Node};

use super::crossfader::{Crossfader, CrossfaderLevel};

pub fn bypass_fader<F, T, A, B, C>(
    node: ProcOnceShare<T, A>,
    wrapper: &dyn Fn(ProcOnceShare<T, A>) -> B,
    level: C,
) -> Crossfader<F, T, ProcOnceShare<T, A>, B, C>
where
    F: 'static,
    T: CrossfaderLevel<F>,
    A: Node<T>,
    B: Node<T>,
    C: Node<F>,
{
    Crossfader::new(node.clone(), wrapper(node), level)
}
