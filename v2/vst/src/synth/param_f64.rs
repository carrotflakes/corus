use corus_v2::nodes::envelope::Envelope;

pub struct ParamF64 {
    pub value: f64,
    pub envelope: Option<(bool, f64, Envelope)>,
    pub lfo: Option<(bool, Lfo)>,
}

pub struct Lfo {
    pub frequency: f64,
    pub amp: f64,
}

#[derive(Clone, Copy)]
pub struct EnvelopeState {
    pub elapsed: f64,
    pub note_off_time: f64,
}

impl ParamF64 {
    pub fn compute(&self, env_state: &EnvelopeState) -> f64 {
        let EnvelopeState {
            elapsed,
            note_off_time,
        } = *env_state;

        self.value
            + if let Some((true, amount, envelope)) = &self.envelope {
                let envelope_level = envelope.compute(elapsed, note_off_time);
                amount * envelope_level
            } else {
                0.0
            }
            + if let Some((true, lfo)) = &self.lfo {
                (elapsed * lfo.frequency * std::f64::consts::TAU).sin() * lfo.amp
            } else {
                0.0
            }
    }
}
