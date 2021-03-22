use crate::{
    core::{
        add::Add, amp::Amp, constant::Constant, placeholder::Placeholder,
        ring_buffer_playback::RingBufferPlayback, ring_buffer_record::RingBufferRecord,
        share::Share, Node,
    },
    signal::C2f64,
};

pub fn delay_fx<A: Node<C2f64> + 'static + Send + Sync>(
    node: A,
    sample_rate: usize,
    delay: f64,
    feedback: f64,
) -> Share<C2f64, RingBufferRecord<C2f64, Placeholder<C2f64, Box<dyn Node<C2f64> + Send + Sync>>>> {
    let mut p = Placeholder::new(None);
    let mut ps = p.setter();
    let buffer = Share::new(RingBufferRecord::new(p, sample_rate));
    unsafe {
        ps.set(Box::new(Add::new(
            node,
            Amp::new(
                RingBufferPlayback::new(Constant::from(delay), buffer.clone()),
                Constant::from(feedback),
            ),
        )) as Box<dyn Node<C2f64> + Send + Sync>);
    }
    buffer
}
