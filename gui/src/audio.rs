use corus::{node::Node, proc_context::ProcContext, signal::C1f32};
use sdl2::audio::AudioCallback;

pub struct Audio {
    node: Box<dyn Node<C1f32> + Send>,
    pub ctx: ProcContext,
}

impl Audio {
    pub fn new(sample_rate: u64, node: Box<dyn Node<C1f32> + Send>) -> Self {
        Self {
            node,
            ctx: ProcContext::new(sample_rate),
        }
    }
}

impl AudioCallback for Audio {
    type Channel = f32;

    fn callback(&mut self, out: &mut [f32]) {
        self.node.lock();
        for x in out.iter_mut() {
            *x = self.ctx.sample(&mut self.node).0[0];
        }
        self.node.unlock();
    }
}
