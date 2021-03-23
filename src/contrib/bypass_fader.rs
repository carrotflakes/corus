use crate::{core::{share::Share, Node}, signal::Signal};

use super::crossfader::{Crossfader, CrossfaderLevel};

pub fn bypass_fader<A, B, C>(
    node: Share<A>,
    wrapper: &dyn Fn(Share<A>) -> B,
    level: C,
) -> Crossfader<Share<A>, B, C>
where
    A: Node,
    B: Node<Output = A::Output>,
    C: Node,
    A::Output: Signal + CrossfaderLevel<C::Output>,
    C::Output: Signal,
{
    Crossfader::new(node.clone(), wrapper(node), level)
}
