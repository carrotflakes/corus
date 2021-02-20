use crate::{node::{Node, accumulator::Accumulator, amp::Amp, controllable::{Controllable, Controller}, map::Map, pan::Pan, param::Param}, signal::{C1f32, C2f32}};

pub fn sine<A: Node<C1f32>, DA: AsMut<A>>(frequency: DA) -> impl Node<C1f32> {
    let acc = Accumulator::new(frequency, 1.0.into());
    Map::new(acc, |v| {
        C1f32::from((v.0[0] * 2.0 * std::f32::consts::PI).sin())
    })
}

pub fn retriggerable_sine<A: Node<C1f32> + 'static, DA: AsMut<A> + 'static>(
    frequency: DA,
) -> (impl Node<C1f32>, impl FnMut(f64)) {
    let acc = Controllable::new(Accumulator::new(frequency, 1.0.into()));
    let mut acc_ctrl = acc.controller();
    (
        Map::new(acc, |v| {
            C1f32::from((v.0[0] * 2.0 * std::f32::consts::PI).sin())
        }),
        move |time: f64| acc_ctrl.lock().set_value_at_time(time, 0.0.into()),
    )
}

pub fn retriggerable_saw<A: Node<C1f32> + 'static, DA: AsMut<A> + 'static>(
    frequency: DA,
) -> (impl Node<C1f32>, impl FnMut(f64)) {
    let acc = Controllable::new(Accumulator::new(frequency, 1.0.into()));
    let mut acc_ctrl = acc.controller();
    (
        Map::new(acc, |v| C1f32::from((v.0[0] - 0.5) * 2.0)),
        move |time: f64| acc_ctrl.lock().set_value_at_time(time, 0.0.into()),
    )
}

pub fn amp_pan<A: Node<C1f32> + 'static, G: Node<C1f32> + 'static, P: Node<C1f32> + 'static>(
    node: impl AsMut<A>,
    gain: impl AsMut<G>,
    pan: impl AsMut<P>,
) -> impl Node<C2f32> {
    Pan::new(Amp::new(node, gain), pan)
}

pub fn controllable_param(initial_value: f32) -> (Controllable<C1f32, Param>, Controller<C1f32, Param>){
    let c = Controllable::new(Param::new());
    let mut ctrl = c.controller();
    ctrl.lock().set_value_at_time(0.0, initial_value);
    (c, ctrl)
}
