use crate::core::{share::Share, Node};

use super::crossfader::{Crossfader, CrossfaderLevel};

pub fn bypass_fader<F, T, A, B, C>(
    node: Share<T, A>,
    wrapper: &dyn Fn(Share<T, A>) -> B,
    level: C,
) -> Crossfader<F, T, Share<T, A>, B, C>
where
    F: 'static,
    T: CrossfaderLevel<F>,
    A: Node<T>,
    B: Node<T>,
    C: Node<F>,
{
    Crossfader::new(node.clone(), wrapper(node), level)
}
