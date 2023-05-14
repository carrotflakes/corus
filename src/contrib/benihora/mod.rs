mod benihora;
mod glottis;
mod tract;

use crate::{
    core::{
        add::Add,
        biquad_filter::{types::BandPass, BiquadFilter, BiquadFilterParams},
        var::Var,
        Node,
    },
    signal::{C1f64, Mono},
    ProcContext,
};

use self::tract::Constriction;

use super::{fn_processor::FnProcessor, perlin_noise, rand::Rand};

type F = f64;

fn simplex1(x: F) -> F {
    perlin_noise(x * 1.2, -x * 0.7, 0.0) as F
}

pub fn make_noise_node() -> Box<dyn Node<Output = f64> + Send + Sync> {
    let node1 = BiquadFilter::new(
        {
            let mut rand = Rand::new(1);
            FnProcessor::new(move || C1f64::from_m(rand.next_f64() * 2.0 - 1.0))
        },
        BiquadFilterParams::new(BandPass, Var::from(500.0), Var::from(0.0), Var::from(2.5)),
    ); // q 0.5
    let node2 = BiquadFilter::new(
        {
            let mut rand = Rand::new(1);
            FnProcessor::new(move || C1f64::from_m(rand.next_f64() * 2.0 - 1.0))
        },
        BiquadFilterParams::new(BandPass, Var::from(1000.0), Var::from(0.0), Var::from(2.5)),
    ); // q 0.5
    Box::new(Add::new(node1, node2))
}

pub struct Benihora {
    benihora: benihora::Benihora,
    node: Box<dyn Node<Output = f64> + Send + Sync>,
}

impl Benihora {
    pub fn new(node: Box<dyn Node<Output = f64> + Send + Sync>, proc_num: usize) -> Self {
        Self {
            benihora: benihora::Benihora::new(proc_num),
            node,
        }
    }
}

impl Node for Benihora {
    type Output = C1f64;

    #[inline]
    fn proc(&mut self, ctx: &ProcContext) -> C1f64 {
        let v = self.node.as_mut().proc(ctx);
        self.benihora.process(ctx.current_time, ctx.sample_rate, v)
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
                self.benihora.tract.calculate_diameter();
            }
            BenihoraEvent::SetFrequency(frequency) => {
                self.benihora.glottis.frequency.set(*frequency);
            }
            BenihoraEvent::SetTenseness(tenseness) => {
                self.benihora.glottis.set_tenseness(*tenseness);
            }
            BenihoraEvent::SetStatus(breath, close) => {
                self.benihora.glottis.breath = *breath;
                self.benihora.glottis.glottis_close = *close;
            }
            BenihoraEvent::SetVibrato(amount, frequency) => {
                self.benihora.glottis.frequency.vibrato_amount = *amount;
                self.benihora.glottis.frequency.vibrato_frequency = *frequency;
            }
        }
    }
}
