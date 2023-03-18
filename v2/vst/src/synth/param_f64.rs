use corus_v2::nodes::envelope::Envelope;

pub struct ParamF64 {
    pub value: f64,
    pub envelope: Option<(f64, Envelope)>,
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

        if let Some((amount, envelope)) = &self.envelope {
            let envelope_level = envelope.compute(elapsed, note_off_time);
            self.value + amount * envelope_level
        } else {
            self.value
        }
    }
}
