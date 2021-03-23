use crate::{
    core::{
        add::Add, all_pass_filter::AllPassFilter, amp::Amp, comb_filter::CombFilter,
        constant::Constant, mix::Mix, share::Share, Node,
    },
    signal::Signal,
};

pub fn schroeder_reverb<N>(
    node: N,
) -> Add<Share<N>, Amp<AllPassFilter<AllPassFilter<Mix<CombFilter<Share<N>>>>>, Constant<f64>>>
where
    N: Node + 'static,
    N::Output: Signal<Float = f64> + From<f64>,
{
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
