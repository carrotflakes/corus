use crate::{
    node::{controllable::Controllable, param::Param},
    signal::C1f32,
};

use super::controllable_param;

#[derive(Debug, Clone)]
pub struct AdsrEnvelope {
    pub a: f32,
    pub d: f32,
    pub s: f32,
    pub r: f32,
}

impl AdsrEnvelope {
    pub fn build(&self) -> (Controllable<C1f32, Param>, impl FnMut(f64), impl FnMut(f64)) {
        let (env, env_ctrl) = controllable_param(0.0);
        (
            env,
            {
                let mut env_ctrl = env_ctrl.clone();
                let a = self.a as f64;
                let d = self.d;
                let a_s = a + self.s as f64;
                move |time| {
                    let mut env = env_ctrl.lock();
                    env.cancel_and_hold_at_time(time);
                    env.set_value_at_time(time, 0.001);
                    env.exponential_ramp_to_value_at_time(time + a, 1.0);
                    env.exponential_ramp_to_value_at_time(time + a_s, d);
                }
            },
            {
                let mut env_ctrl = env_ctrl.clone();
                let r = self.r as f64;
                move |time| {
                    let mut env = env_ctrl.lock();
                    env.cancel_and_hold_at_time(time);
                    env.exponential_ramp_to_value_at_time(time + r, 0.001);
                    // env.set_target_at_time(time, 0.0, r);
                }
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct ArEnvelope {
    pub a: f32,
    pub r: f32,
}

impl ArEnvelope {
    pub fn build(&self) -> (Controllable<C1f32, Param>, impl FnMut(f64), impl FnMut(f64)) {
        let (env, env_ctrl) = controllable_param(0.0);
        (
            env,
            {
                let mut env_ctrl = env_ctrl.clone();
                let a = self.a as f64;
                let a_r = a + self.r as f64;
                move |time| {
                    let mut env = env_ctrl.lock();
                    env.cancel_and_hold_at_time(time);
                    env.set_value_at_time(time, 0.001);
                    env.exponential_ramp_to_value_at_time(time + a, 1.0);
                    env.exponential_ramp_to_value_at_time(time + a_r, 0.001);
                }
            },
            {
                move |_| {
                }
            },
        )
    }
}
