use crate::{
    core::{
        add::Add, amp::Amp, placeholder::Placeholder, ring_buffer_playback::RingBufferPlayback,
        ring_buffer_record::RingBufferRecord, share::Share, var::Var, Node,
    },
    signal::Signal,
};

pub fn delay_fx<
    S: Signal<Float = f64> + Sync + Send,
    A: Node<Output = S> + 'static + Send + Sync,
>(
    node: A,
    sample_rate: usize,
    delay: f64,
    feedback: f64,
) -> Share<RingBufferRecord<Placeholder<Box<dyn Node<Output = S> + Send + Sync>>>> {
    let mut p = Placeholder::new(None);
    let mut ps = p.setter();
    let buffer = Share::new(RingBufferRecord::new(
        p,
        (sample_rate as f64 * delay) as usize + 1,
    ));
    unsafe {
        ps.set(Box::new(Add::new(
            node,
            Amp::new(
                RingBufferPlayback::new(Var::from(delay), buffer.clone()),
                Var::from(feedback),
            ),
        )) as Box<dyn Node<Output = S> + Send + Sync>);
    }
    buffer
}
