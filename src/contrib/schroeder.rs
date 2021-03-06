use std::ops::{Mul, Neg};

use crate::core::{
    add::Add, all_pass_filter::AllPassFilter, amp::Amp, comb_filter::CombFilter,
    constant::Constant, mix::Mix, share::Share, Node,
};

pub fn schroeder_reverb<
    T: Clone
        + 'static
        + Default
        + Mul<Output = T>
        + std::ops::Add<Output = T>
        + Neg<Output = T>
        + From<f64>,
    N: Node<T> + 'static,
>(
    node: N,
) -> Add<
    T,
    Share<T, N>,
    Amp<
        T,
        AllPassFilter<T, AllPassFilter<T, Mix<T, CombFilter<T, Share<T, N>>>>>,
        Constant<T>,
    >,
> {
    let node = Share::new(node);
    let nodes: Vec<_> = (0..4)
        .map(|i| {
            CombFilter::new(
                node.clone(),
                0.03 + 0.0041 * i as f64,
                (0.6 + i as f64 * 0.02).into(),
            )
        })
        .collect();
    let mix = Mix::new(nodes);
    let rev = AllPassFilter::new(mix, 0.0015, 0.85.into());
    let rev = AllPassFilter::new(rev, 0.0133, 0.78.into());
    let rev = Amp::new(rev, Constant::from(0.7 / 4.0));
    Add::new(node, rev)
}
