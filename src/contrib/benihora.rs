use crate::{core::Node, signal::C1f64, ProcContext};

use benihora::Constriction;

type F = f64;

pub struct Benihora {
    benihora: benihora::BenihoraManaged,
}

impl Benihora {
    pub fn new(proc_num: usize, sample_rate: f64) -> Self {
        Self {
            benihora: benihora::BenihoraManaged::new(proc_num, sample_rate),
        }
    }
}

impl Node for Benihora {
    type Output = C1f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        self.benihora.process(ctx.current_time)
    }

    fn lock(&mut self, _ctx: &ProcContext) {}

    fn unlock(&mut self) {}
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
                self.benihora.benihora.tract.mouth.tongue = self
                    .benihora
                    .benihora
                    .tract
                    .mouth
                    .tangue_clamp(*index, *diameter);
                self.benihora.benihora.tract.calculate_diameter();
            }
            BenihoraEvent::SetOtherConstrictions(new_ocs) => {
                let ocs = &mut self.benihora.benihora.tract.mouth.other_constrictions;
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
                self.benihora.benihora.tract.calculate_diameter();
            }
            BenihoraEvent::SetVelum(velum) => {
                self.benihora.benihora.tract.nose.velum_target = *velum;
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
