use crate::{node::{Node, add::Add, all_pass_filter::AllPassFilter, amp::Amp, comb_filter::CombFilter, constant::Constant, mix::Mix, proc_once_share::ProcOnceShare}, signal::C1f32};

pub fn schroeder_reverb<N: Node<C1f32> + 'static, DN: AsMut<N> + 'static>(
    node: DN,
) -> Add<
    C1f32,
    ProcOnceShare<C1f32, N, DN>,
    Amp<
        C1f32,
        AllPassFilter<
            C1f32,
            AllPassFilter<
                C1f32,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
            >,
            AllPassFilter<
                C1f32,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
            >,
        >,
        Constant<C1f32>,
        AllPassFilter<
            C1f32,
            AllPassFilter<
                C1f32,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
            >,
            AllPassFilter<
                C1f32,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
            >,
        >,
        Constant<C1f32>,
    >,
    ProcOnceShare<C1f32, N, DN>,
    Amp<
        C1f32,
        AllPassFilter<
            C1f32,
            AllPassFilter<
                C1f32,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
            >,
            AllPassFilter<
                C1f32,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
            >,
        >,
        Constant<C1f32>,
        AllPassFilter<
            C1f32,
            AllPassFilter<
                C1f32,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
            >,
            AllPassFilter<
                C1f32,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
                Mix<C1f32, Box<dyn Node<C1f32>>>,
            >,
        >,
        Constant<C1f32>,
    >,
> {
    let node = ProcOnceShare::new(node);
    let nodes: Vec<_> = (0..4)
        .map(|i| {
            Box::new(CombFilter::new(
                node.clone(),
                0.03 + 0.0041 * i as f32,
                (0.6 + i as f32 * 0.02).into(),
            )) as Box<dyn Node<C1f32>>
        })
        .collect();
    let mix = Mix::new(nodes);
    let rev = AllPassFilter::new(mix, 0.0015, 0.85.into());
    let rev = AllPassFilter::new(rev, 0.0133, 0.78.into());
    let rev = Amp::new(rev, Constant::from(0.7 / 4.0));
    Add::new(node, rev)
}
