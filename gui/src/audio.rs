use corus::{
    signal::{C1f64, Mono},
    time::Sample,
    Node, ProcContext,
};
use sdl2::audio::AudioCallback;

pub struct Audio {
    node: Box<dyn Node<C1f64> + Send>,
    pub ctx: ProcContext,
}

impl Audio {
    pub fn new(sample_rate: u64, node: Box<dyn Node<C1f64> + Send>) -> Self {
        Self {
            node,
            ctx: ProcContext::new(sample_rate),
        }
    }
}

impl AudioCallback for Audio {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        let mut s = self.ctx.lock(&mut self.node, Sample(out.len() as u64));
        for x in out.iter_mut() {
            *x = s.next().unwrap().get_m() as f32;
        }
    }
}
