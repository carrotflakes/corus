mod glottis;
mod tract;

use glottis::Glottis;
use tract::{Tract, Constriction};

use crate::{
    core::Node,
    proc_context::ProcContext,
    signal::{C1f64, Mono},
};

type F = f64;

pub struct Benihora<A: Node<Output = f64> + 'static, B: Node<Output = f64> + 'static> {
    noise: Box<dyn Node<Output = f64> + Send + Sync>,
    glottis: Glottis<A, B>,
    tract: Tract,
    block_time: f64,         // sec
    block_updated_time: f64, // sec
    proc_num: usize,
}

impl<A: Node<Output = f64> + 'static, B: Node<Output = f64> + 'static> Benihora<A, B> {
    pub fn new(frequency: A, tenseness: B, proc_num: usize) -> Self {
        Self {
            glottis: Glottis::new(frequency, tenseness),
            tract: Tract::new(),
            noise: super::benihora::make_noise_node(),
            block_time: 0.04,
            block_updated_time: 0.0,
            proc_num,
        }
    }
}

impl<A: Node<Output = f64> + 'static, B: Node<Output = f64> + 'static> Node for Benihora<A, B> {
    type Output = C1f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        if self.block_updated_time + self.block_time <= ctx.current_time {
            self.block_updated_time += self.block_time;
            self.glottis.update_block(self.block_time);
            self.tract.update_block(self.block_time);
        }

        let v = self.noise.as_mut().proc(ctx);
        let lambda = (ctx.current_time - self.block_updated_time) / self.block_time; // TODO: lambdaなくしたい
        let (glottal_output, turbulence_noise) = self.glottis.run_step(
            ctx.current_time,
            ctx.sample_rate as usize,
            lambda,
            v.get_m() as f64,
        );
        let glottal_output = glottal_output + v * 1.0e-20; // tract に 0.0 を渡すと何故か遅くなるので僅かにノイズを混ぜる
        let mut vocal_out = 0.0;
        for i in 0..self.proc_num {
            let time =
                ctx.current_time + (i as f64 / self.proc_num as f64) / ctx.sample_rate as f64;
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
        self.noise.lock(ctx);
    }

    fn unlock(&mut self) {
        self.noise.unlock();
    }
}

pub enum BenihoraEvent {
    MoveTangue(F, F),
    SetOtherConstrictions(Vec<(F, F)>),
    SetStatus(bool, bool),
}

impl<A: Node<Output = f64> + 'static, B: Node<Output = f64> + 'static> crate::EventListener<BenihoraEvent> for Benihora<A, B> {
    #[inline]
    fn apply_event(&mut self, time: f64, event: &BenihoraEvent) {
        match event {
            BenihoraEvent::MoveTangue(index, diameter) => {
                self.tract.mouth.tongue = self.tract.mouth.tangue_clamp(*index, *diameter);
                self.tract.calculate_diameter();
            }
            BenihoraEvent::SetOtherConstrictions(new_ocs) => {
                let ocs = &mut self.tract.mouth.other_constrictions;
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
                self.tract.calculate_diameter();
            }
            BenihoraEvent::SetStatus(breath, close) => {
                self.glottis.breath = *breath;
                self.glottis.glottis_close = *close;
            }
        }
    }
}

fn simplex1(x: f64) -> f64 {
    crate::contrib::perlin_noise::perlin_noise(x * 1.2, -x * 0.7, 0.0) as f64
}
