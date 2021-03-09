use crate::{
    core::Node,
    proc_context::ProcContext,
    signal::{C1f64, Mono},
};

use super::{
    glottis::Glottis,
    tract::{Constriction, Tract},
    F,
};

pub struct Benihora {
    node: Box<dyn Node<C1f64> + Send + Sync>,
    glottis: Glottis,
    tract: Tract,
    block_time: f64,         // sec
    block_updated_time: f64, // sec
    proc_num: usize,
}

impl Benihora {
    pub fn new(node: Box<dyn Node<C1f64> + Send + Sync>, proc_num: usize) -> Self {
        Self {
            glottis: Glottis::new(),
            tract: Tract::new(),
            node,
            block_time: 0.04,
            block_updated_time: 0.0,
            proc_num,
        }
    }
}

impl Node<C1f64> for Benihora {
    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        if self.block_updated_time + self.block_time <= ctx.current_time {
            self.block_updated_time += self.block_time;
            self.glottis.update_block(ctx.current_time, self.block_time);
            self.tract.update_block(self.block_time);
        }

        let v = self.node.as_mut().proc(ctx);
        let lambda = (ctx.current_time - self.block_updated_time) / self.block_time; // TODO: lambdaなくしたい
        let glottal_output = self.glottis.run_step(
            ctx.current_time,
            ctx.sample_rate as usize,
            lambda,
            v.get_m() as F,
        ) + v * 1.0e-20; // tract に 0.0 を渡すと何故か遅くなるので僅かにノイズを混ぜる
        let noise_mod = self.glottis.get_noise_modulator(lambda);
        let turbulence_noise = v.get_m() as F * noise_mod; // v.0[1] is better...
        let mut vocal_out = 0.0;
        for i in 0..self.proc_num {
            let time = ctx.current_time + (i as f64 / self.proc_num as f64) / ctx.sample_rate as f64;
            vocal_out += self.tract.run_step(
                time,
                glottal_output,
                turbulence_noise,
                (time - self.block_updated_time) / self.block_time,
                ctx.sample_rate as usize * self.proc_num,
            );
        }
        (vocal_out / self.proc_num as f64).into()
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.node.lock(ctx);
    }

    fn unlock(&mut self) {
        self.node.unlock();
    }
}

pub enum BenihoraEvent {
    MoveTangue(F, F),
    SetOtherConstrictions(Vec<(F, F)>),
    SetFrequency(F),
    SetTenseness(F),
    SetStatus(bool, bool),
    SetVibrato(F, F),
}

impl crate::Event for BenihoraEvent {
    type Target = Benihora;

    fn dispatch(&self, time: f64, node: &mut Self::Target) {
        match self {
            BenihoraEvent::MoveTangue(index, diameter) => {
                node.tract.mouth.tongue = node.tract.mouth.tangue_clamp(*index, *diameter);
                node.tract.calculate_diameter();
            }
            BenihoraEvent::SetOtherConstrictions(new_ocs) => {
                let ocs = &mut node.tract.mouth.other_constrictions;
                for c in new_ocs.iter() {
                    if ocs
                        .iter()
                        .find(|x| x.index == c.0 && x.diameter == c.1 && x.end_time.is_none())
                        .is_none()
                    {
                        ocs.push(Constriction {
                            index: c.0,
                            diameter: c.1,
                            start_time: time,
                            end_time: None,
                        });
                    }
                }
                for c in ocs.iter_mut() {
                    if c.end_time.is_none()
                        && new_ocs
                            .iter()
                            .find(|x| c.index == x.0 && c.diameter == x.1)
                            .is_none()
                    {
                        c.end_time = Some(time);
                    }
                }
                node.tract.calculate_diameter();
            }
            BenihoraEvent::SetFrequency(frequency) => {
                node.glottis.frequency.set(*frequency);
            }
            BenihoraEvent::SetTenseness(tenseness) => {
                node.glottis.set_tenseness(*tenseness);
            }
            BenihoraEvent::SetStatus(breath, close) => {
                node.glottis.breath = *breath;
                node.glottis.glottis_close = *close;
            }
            BenihoraEvent::SetVibrato(amount, frequency) => {
                node.glottis.frequency.vibrato_amount = *amount;
                node.glottis.frequency.vibrato_frequency = *frequency;
            }
        }
    }
}
