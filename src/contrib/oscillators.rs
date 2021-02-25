use crate::{
    core::{accumulator::Accumulator, controllable::Controllable, map::Map, Node},
    signal::{C1f64, Mono},
};

pub fn sine<A: Node<C1f64>, DA: AsMut<A>>(frequency: DA) -> impl Node<C1f64> {
    let acc = Accumulator::new(frequency, 1.0.into());
    Map::new(acc, |v| {
        C1f64::from((v.get_m() * 2.0 * std::f64::consts::PI).sin())
    })
}

pub fn retriggerable_sine<A: Node<C1f64> + 'static, DA: AsMut<A> + 'static>(
    frequency: DA,
) -> (impl Node<C1f64>, impl FnMut(f64)) {
    let acc = Controllable::new(Accumulator::new(frequency, 1.0.into()));
    let mut acc_ctrl = acc.controller();
    (
        Map::new(acc, |v| {
            C1f64::from((v.get_m() * 2.0 * std::f64::consts::PI).sin())
        }),
        move |time: f64| acc_ctrl.lock().set_value_at_time(time, 0.0.into()),
    )
}

pub fn retriggerable_saw<A: Node<C1f64> + 'static, DA: AsMut<A> + 'static>(
    frequency: DA,
) -> (impl Node<C1f64>, impl FnMut(f64)) {
    let acc = Controllable::new(Accumulator::new(frequency, 1.0.into()));
    let mut acc_ctrl = acc.controller();
    (
        Map::new(acc, |v| C1f64::from((v.get_m() - 0.5) * 2.0)),
        move |time: f64| acc_ctrl.lock().set_value_at_time(time, 0.0.into()),
    )
}

pub fn resetable_acc<A: Node<C1f64> + 'static, DA: AsMut<A> + 'static>(
    frequency: DA,
) -> (
    Controllable<C1f64, Accumulator<C1f64, A, DA>>,
    impl FnMut(f64, f64),
) {
    let acc = Controllable::new(Accumulator::new(frequency, C1f64::from(1.0)));
    let mut acc_ctrl = acc.controller();
    (acc, move |time: f64, value: f64| {
        acc_ctrl.lock().set_value_at_time(time, value.into())
    })
}
