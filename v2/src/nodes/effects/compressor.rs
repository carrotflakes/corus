use num_traits::{Float, FromPrimitive};

use crate::{signal::float_array::FloatArray, ProcessContext};

pub struct Compressor<F: Float> {
    scale: F,
}

pub struct Param<F: Float> {
    pub threshold: F,
    pub ratio: F, // 0.0 ~ 1.0
    pub attack: F, // in seconds
    pub release: F, // in seconds
    pub gain: F,
}

impl<F: Float> Compressor<F> {
    pub fn new() -> Self {
        Self { scale: F::one() }
    }
}

impl<F: Float + FromPrimitive> Compressor<F> {
    pub fn process<const N: usize>(
        &mut self,
        param: &Param<F>,
        ctx: &ProcessContext,
        x: FloatArray<N, F>,
    ) -> FloatArray<N, F> {
        let level = x
            .iter()
            .map(|x| x.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap();
        let desire_scale = if param.threshold < level {
            ((level - param.threshold) * param.ratio + param.threshold) / level
        } else {
            F::one()
        };
        if desire_scale < self.scale {
            let attack = F::from_f64(ctx.dtime()).unwrap() / param.attack;
            self.scale = self.scale - attack;
        } else {
            let release = F::from_f64(ctx.dtime()).unwrap() / param.release;
            self.scale = self.scale + release;
        }
        x * self.scale * param.gain
    }
}
