use crate::{core::Node, signal::C1f64, ProcContext};

type F = f64;

pub struct Benihora {
    benihora: benihora::BenihoraManaged,
}

impl Benihora {
    pub fn new(sound_speed: f64, sample_rate: f64) -> Self {
        Self {
            benihora: benihora::BenihoraManaged::new(sound_speed, sample_rate, 0),
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
    MoveTongue(F, F),
    SetOtherConstrictions(Vec<(F, F)>),
    SetVelum(F),
    SetFrequency(F),
    SetTenseness(F),
    SetStatus(bool),
    SetVibrato(F, F),
}

impl crate::EventListener<BenihoraEvent> for Benihora {
    #[inline]
    fn apply_event(&mut self, _: f64, event: &BenihoraEvent) {
        match event {
            BenihoraEvent::MoveTongue(index, diameter) => {
                self.benihora.benihora.tract.source.tongue = self
                    .benihora
                    .benihora
                    .tract
                    .source
                    .tongue_clamp(*index, *diameter);
                self.benihora.benihora.tract.update_diameter();
            }
            BenihoraEvent::SetOtherConstrictions(new_ocs) => {
                self.benihora.benihora.tract.source.other_constrictions = new_ocs.clone();
                self.benihora.benihora.tract.update_diameter();
            }
            BenihoraEvent::SetVelum(velum) => {
                self.benihora.benihora.tract.set_velum_target(*velum);
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
