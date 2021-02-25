use std::ops::{Mul, Neg};

use crate::core::{
    add::Add, all_pass_filter::AllPassFilter, amp::Amp, comb_filter::CombFilter,
    constant::Constant, mix::Mix, proc_once_share::ProcOnceShare, Node,
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
    DN: AsMut<N> + 'static,
>(
    node: DN,
) -> Add<
    T,
    ProcOnceShare<T, N, DN>,
    Amp<
        T,
        AllPassFilter<
            T,
            AllPassFilter<T, Mix<T, Box<dyn Node<T>>>, Mix<T, Box<dyn Node<T>>>>,
            AllPassFilter<T, Mix<T, Box<dyn Node<T>>>, Mix<T, Box<dyn Node<T>>>>,
        >,
        Constant<T>,
        AllPassFilter<
            T,
            AllPassFilter<T, Mix<T, Box<dyn Node<T>>>, Mix<T, Box<dyn Node<T>>>>,
            AllPassFilter<T, Mix<T, Box<dyn Node<T>>>, Mix<T, Box<dyn Node<T>>>>,
        >,
        Constant<T>,
    >,
    ProcOnceShare<T, N, DN>,
    Amp<
        T,
        AllPassFilter<
            T,
            AllPassFilter<T, Mix<T, Box<dyn Node<T>>>, Mix<T, Box<dyn Node<T>>>>,
            AllPassFilter<T, Mix<T, Box<dyn Node<T>>>, Mix<T, Box<dyn Node<T>>>>,
        >,
        Constant<T>,
        AllPassFilter<
            T,
            AllPassFilter<T, Mix<T, Box<dyn Node<T>>>, Mix<T, Box<dyn Node<T>>>>,
            AllPassFilter<T, Mix<T, Box<dyn Node<T>>>, Mix<T, Box<dyn Node<T>>>>,
        >,
        Constant<T>,
    >,
> {
    let node = ProcOnceShare::new(node);
    let nodes: Vec<_> = (0..4)
        .map(|i| {
            Box::new(CombFilter::new(
                node.clone(),
                0.03 + 0.0041 * i as f64,
                (0.6 + i as f64 * 0.02).into(),
            )) as Box<dyn Node<T>>
        })
        .collect();
    let mix = Mix::new(nodes);
    let rev = AllPassFilter::new(mix, 0.0015, 0.85.into());
    let rev = AllPassFilter::new(rev, 0.0133, 0.78.into());
    let rev = Amp::new(rev, Constant::from(0.7 / 4.0));
    Add::new(node, rev)
}
