use std::marker::PhantomData;

use crate::{core::param3::ParamEventScheduleNode, signal::Mono};

type F = f64;

pub trait EnvelopeGenerator<T: Mono<F>> {
    fn generate(
        &self,
    ) -> (
        ParamEventScheduleNode<F>,
        Box<dyn FnMut(f64) + Send + Sync>,
        Box<dyn FnMut(f64) + Send + Sync>,
    );
}

#[derive(Debug, Clone)]
pub struct AdsrEnvelope<F: 'static + Clone + Default, T: Mono<F>> {
    pub a: f64,
    pub d: F,
    pub s: f64,
    pub r: f64,
    _t: PhantomData<T>,
}

impl<T: Mono<F> + Send + Sync> AdsrEnvelope<F, T> {
    pub fn new(a: f64, d: F, s: f64, r: f64) -> Self {
        assert!(0.0 < d);
        Self { a, d, s, r, _t: Default::default() }
    }

    pub fn build(
        &self,
    ) -> (
        ParamEventScheduleNode<F>,
        impl FnMut(f64) + Send + Sync,
        impl FnMut(f64) + Send + Sync,
    ) {
        let mut env = ParamEventScheduleNode::new();
        let env_ctl = env.get_scheduler();
        (
            env,
            {
                let env_ctl = env_ctl.clone();
                let a = self.a as f64;
                let d = self.d.clone();
                let a_s = a + self.s as f64;
                move |time| {
                    let mut env = env_ctl.lock().unwrap();
                    env.cancel_and_hold_at_time(time);
                    env.set_value_at_time(time, 0.001.into());
                    env.exponential_ramp_to_value_at_time(time + a, 1.0.into());
                    env.exponential_ramp_to_value_at_time(time + a_s, d.clone());
                }
            },
            {
                let env_ctl = env_ctl.clone();
                let r = self.r as f64;
                move |time| {
                    let mut env = env_ctl.lock().unwrap();
                    env.cancel_and_hold_at_time(time);
                    env.exponential_ramp_to_value_at_time(time + r, 0.001.into());
                    // env.set_target_at_time(time, 0.0, r);
                }
            },
        )
    }
}

#[derive(Debug, Clone)]
pub struct ArEnvelope<T: Mono<F> + Send + Sync> {
    pub a: f64,
    pub r: f64,
    _t: PhantomData<T>,
}

impl<T: Mono<F> + Send + Sync> ArEnvelope<T> {
    pub fn new(a: f64, r: f64) -> Self {
        Self { a, r, _t: Default::default() }
    }
    pub fn build(
        &self,
    ) -> (
        ParamEventScheduleNode<F>,
        impl FnMut(f64) + Send + Sync,
        impl FnMut(f64) + Send + Sync,
    ) {
        let mut env = ParamEventScheduleNode::new();
        let env_ctl = env.get_scheduler();
        (
            env,
            {
                let env_ctl = env_ctl.clone();
                let a = self.a as f64;
                let a_r = a + self.r as f64;
                move |time| {
                    let mut env = env_ctl.lock().unwrap();
                    env.cancel_and_hold_at_time(time);
                    env.set_value_at_time(time, 0.001.into());
                    env.exponential_ramp_to_value_at_time(time + a, 1.0.into());
                    env.exponential_ramp_to_value_at_time(time + a_r, 0.001.into());
                }
            },
            { move |_| {} },
        )
    }
}

impl<T: Mono<F> + Send + Sync> EnvelopeGenerator<T> for AdsrEnvelope<F, T> {
    fn generate(
        &self,
    ) -> (
        ParamEventScheduleNode<F>,
        Box<dyn FnMut(f64) + Send + Sync>,
        Box<dyn FnMut(f64) + Send + Sync>,
    ) {
        let (c, on, off) = self.build();
        (c, Box::new(on), Box::new(off))
    }
}

impl<T: Mono<F> + Send + Sync> EnvelopeGenerator<T> for ArEnvelope<T> {
    fn generate(
        &self,
    ) -> (
        ParamEventScheduleNode<F>,
        Box<dyn FnMut(f64) + Send + Sync>,
        Box<dyn FnMut(f64) + Send + Sync>,
    ) {
        let (c, on, off) = self.build();
        (c, Box::new(on), Box::new(off))
    }
}
