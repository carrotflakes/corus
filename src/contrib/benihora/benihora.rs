use crate::{
    node::Node,
    proc_context::ProcContext,
    signal::{C1f64, C2f32},
};

use super::{glottis::Glottis, tract::Tract, F};

pub struct Benihora {
    node: Box<dyn Node<C2f32>>,
    glottis: Glottis,
    tract: Tract,
    block_time: f64,         // sec
    block_updated_time: f64, // sec
    pub twice: bool,
}

impl Benihora {
    pub fn new(node: Box<dyn Node<C2f32>>) -> Self {
        Self {
            glottis: Glottis::new(),
            tract: Tract::new(),
            node,
            block_time: 0.04,
            block_updated_time: 0.0,
            twice: true,
        }
    }
}

impl Node<C1f64> for Benihora {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        if self.block_updated_time + self.block_time <= ctx.time {
            self.block_updated_time += self.block_time;
            self.glottis.update_block(self.block_time);
            self.tract.update_block(self.block_time);
        }

        let v = self.node.as_mut().proc(ctx);
        let lambda1 = (ctx.time - self.block_updated_time) / self.block_time;
        let lambda2 =
            (ctx.time - self.block_updated_time + 0.5 / ctx.sample_rate as f64) / self.block_time;
        let glottal_output = self
            .glottis
            .run_step(ctx.sample_rate as usize, lambda1, v.0[0] as F);
        let noise_mod = self.glottis.get_noise_modulator();
        if self.twice {
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
            self.tract
                .run_step(
                    glottal_output,
                    v.0[1] as F,
                    lambda1,
                    ctx.sample_rate as usize,
                    noise_mod,
                )
                .into()
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
