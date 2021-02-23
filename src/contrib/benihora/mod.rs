mod glottis;
mod tract;

use std::usize;

use glottis::Glottis;
use tract::Tract;

use crate::{
    node::{
        add::Add,
        biquad_filter::{BandPass, BiquadFilter},
        constant::Constant,
        Node,
    },
    proc_context::ProcContext,
    signal::{C1f64, C2f32},
};

use super::{fn_processor::FnProcessor, perlin_noise, rand::Rand};

type F = f64;

pub struct Benihora {
    node: Box<dyn Node<C2f32>>,
    glottis: Glottis,
    tract: Tract,
    block_time: f64, // sec
    block_updated_time: f64, // sec
}

impl Benihora {
    pub fn new(node: Box<dyn Node<C2f32>>) -> Self {
        Self {
            glottis: Glottis::new(),
            tract: Tract::new(),
            node,
            block_time: 0.04,
            block_updated_time: 0.0,
        }
    }
}

impl Node<C1f64> for Benihora {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        if self.block_updated_time + self.block_time <= ctx.time {
            self.block_updated_time += self.block_time;
            self.glottis
                .update_block(self.block_time);
            self.tract
                .update_block(self.block_time);
        }

        let v = self.node.as_mut().proc(ctx);
        let lambda1 = (ctx.time - self.block_updated_time) / self.block_time;
        let lambda2 = (ctx.time - self.block_updated_time + 0.5 / ctx.sample_rate as f64) / self.block_time;
        let glottal_output = self
            .glottis
            .run_step(ctx.sample_rate as usize, lambda1, v.0[0] as F);
        let noise_mod = self.glottis.get_noise_modulator();
        if true {
            let mut vocal_out = 0.0;
            vocal_out += self.tract.run_step(
                glottal_output,
                v.0[1] as F,
                lambda1,
                ctx.sample_rate as usize * 2,
                noise_mod,
            );
            vocal_out += self.tract.run_step(
                glottal_output,
                v.0[1] as F,
                lambda2,
                ctx.sample_rate as usize * 2,
                noise_mod,
            );

            (vocal_out * 0.5).into()
        } else {
            self.tract.run_step(
                glottal_output,
                v.0[1] as F,
                lambda1,
                ctx.sample_rate as usize * 2,
                noise_mod,
            ).into()
        }
    }

    fn lock(&mut self) {
        self.node.lock();
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

impl AsMut<Self> for Benihora {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

fn simplex1(x: F) -> F {
    perlin_noise(x * 1.2, -x * 0.7, 0.0) as F
}

pub fn make_noise_node() -> Box<dyn Node<C2f32>> {
    let node1 = BiquadFilter::new(
        BandPass,
        {
            let mut rand = Rand::new(1);
            FnProcessor::new(move || {
                C2f32([rand.next_f32() * 2.0 - 1.0, rand.next_f32() * 2.0 - 1.0])
            })
        },
        Constant::from(500.0),
        Constant::from(0.0),
        Constant::from(2.5),
    ); // q 0.5
    let node2 = BiquadFilter::new(
        BandPass,
        {
            let mut rand = Rand::new(1);
            FnProcessor::new(move || {
                C2f32([rand.next_f32() * 2.0 - 1.0, rand.next_f32() * 2.0 - 1.0])
            })
        },
        Constant::from(1000.0),
        Constant::from(0.0),
        Constant::from(2.5),
    ); // q 0.5
    Box::new(Add::new(node1, node2))
}
