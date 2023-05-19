use crate::{
    core::{
        biquad_filter::{types::BandPass, BiquadFilter, BiquadFilterParams},
        var::Var,
        Node,
    },
    signal::{C1f64, Mono},
    ProcContext,
};

use benihora::Constriction;

use super::{fn_processor::FnProcessor, rand::Rand};

type F = f64;

pub fn make_noise_node(seed: u32, frequency: f64) -> Box<dyn Node<Output = f64> + Send + Sync> {
    let node = BiquadFilter::new(
        {
            let mut rand = Rand::new(seed);
            FnProcessor::new(move || C1f64::from_m(rand.next_f64() * 2.0 - 1.0))
        },
        BiquadFilterParams::new(
            BandPass,
            Var::from(frequency),
            Var::from(0.0),
            Var::from(0.5),
        ),
    );
    Box::new(node)
}

pub struct Benihora {
    benihora: benihora::Benihora,
    aspiration_noise: Box<dyn Node<Output = f64> + Send + Sync>,
    fricative_noise: Box<dyn Node<Output = f64> + Send + Sync>,
}

impl Benihora {
    pub fn new(proc_num: usize) -> Self {
        Self {
            benihora: benihora::Benihora::new(proc_num),
            aspiration_noise: make_noise_node(1, 500.0),
            fricative_noise: make_noise_node(2, 1000.0),
        }
    }
}

impl Node for Benihora {
    type Output = C1f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let aspiration_noise = self.aspiration_noise.proc(ctx);
        let fricative_noise = self.fricative_noise.proc(ctx);

        self.benihora.process(
            ctx.current_time,
            ctx.sample_rate,
            aspiration_noise,
            fricative_noise,
        )
    }

    fn lock(&mut self, ctx: &ProcContext) {
        self.aspiration_noise.lock(ctx);
        self.fricative_noise.lock(ctx);
    }

    fn unlock(&mut self) {
        self.aspiration_noise.unlock();
        self.fricative_noise.unlock();
    }
}

pub enum BenihoraEvent {
    MoveTangue(F, F),
    SetOtherConstrictions(Vec<(F, F)>),
    SetVelum(F),
    SetFrequency(F),
    SetTenseness(F),
    SetStatus(bool),
    SetVibrato(F, F),
}

impl crate::EventListener<BenihoraEvent> for Benihora {
    #[inline]
    fn apply_event(&mut self, time: f64, event: &BenihoraEvent) {
        match event {
            BenihoraEvent::MoveTangue(index, diameter) => {
                self.benihora.tract.mouth.tongue =
                    self.benihora.tract.mouth.tangue_clamp(*index, *diameter);
                self.benihora.tract.calculate_diameter();
            }
            BenihoraEvent::SetOtherConstrictions(new_ocs) => {
                let ocs = &mut self.benihora.tract.mouth.other_constrictions;
                for c in new_ocs.iter() {
                    if !ocs
                        .iter()
                        .any(|x| x.index == c.0 && x.diameter == c.1 && x.end_time.is_none())
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
                self.benihora.tract.calculate_diameter();
            }
            BenihoraEvent::SetVelum(velum) => {
                self.benihora.tract.nose.velum_target = *velum;
            }
            BenihoraEvent::SetFrequency(frequency) => {
                self.benihora.frequency.set(*frequency);
            }
            BenihoraEvent::SetTenseness(tenseness) => {
                self.benihora.set_tenseness(*tenseness);
            }
            BenihoraEvent::SetStatus(sound) => {
                self.benihora.sound = *sound;
            }
            BenihoraEvent::SetVibrato(amount, frequency) => {
                self.benihora.frequency.vibrato_amount = *amount;
                self.benihora.frequency.vibrato_frequency = *frequency;
            }
        }
    }
}
