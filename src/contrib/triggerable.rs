pub trait Triggerable<Payload> {
    fn bang(&mut self, time: f64, payload: Payload);
}
