use crate::{
    core::{
        accumulator::{Accumulator, SetValueAtTime},
        controllable::Controllable,
        map::Map,
        Node,
    },
    signal::{C1f64, Mono},
    EventControlInplace, EventPusher,
};

pub fn sine<A: Node<C1f64>>(frequency: A) -> impl Node<C1f64> {
    let acc = Accumulator::new(frequency, 1.0.into());
    Map::new(acc, |v| {
        C1f64::from((v.get_m() * 2.0 * std::f64::consts::PI).sin())
    })
}

pub fn square<A: Node<C1f64>>(frequency: A, pwm: f64) -> impl Node<C1f64> {
    let acc = Accumulator::new(frequency, 1.0.into());
    Map::new(acc, move |v| C1f64::from(if v.get_m() < pwm { -1.0 } else { 1.0 }))
}

pub fn retriggerable_sine<A: Node<C1f64> + 'static>(
    frequency: A,
) -> (impl Node<C1f64>, impl FnMut(f64)) {
    let acc = Controllable::new(EventControlInplace::new(Accumulator::new(
        frequency,
        1.0.into(),
    )));
    let mut acc_ctrl = acc.controller();
    (
        Map::new(acc, |v: f64| {
            C1f64::from((v.get_m() * 2.0 * std::f64::consts::PI).sin())
        }),
        move |time: f64| acc_ctrl.lock().push_event(time, SetValueAtTime::new(0.0)),
    )
}

pub fn retriggerable_saw<A: Node<C1f64> + 'static>(
    frequency: A,
) -> (impl Node<C1f64>, impl FnMut(f64)) {
    let acc = Controllable::new(EventControlInplace::new(Accumulator::new(
        frequency,
        1.0.into(),
    )));
    let mut acc_ctrl = acc.controller();
    (
        Map::new(acc, |v: f64| C1f64::from((v.get_m() - 0.5) * 2.0)),
        move |time: f64| acc_ctrl.lock().push_event(time, SetValueAtTime::new(0.0)),
    )
}

pub fn resetable_acc<A: Node<C1f64> + 'static>(
    frequency: A,
) -> (
    Controllable<C1f64, EventControlInplace<SetValueAtTime<C1f64, A>, Accumulator<C1f64, A>>>,
    impl FnMut(f64, f64),
) {
    let acc = Controllable::new(EventControlInplace::new(Accumulator::new(
        frequency,
        C1f64::from(1.0),
    )));
    let mut acc_ctrl = acc.controller();
    (acc, move |time: f64, value: f64| {
        acc_ctrl.lock().push_event(time, SetValueAtTime::new(value))
    })
}
