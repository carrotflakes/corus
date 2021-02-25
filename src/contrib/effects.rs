use crate::{
    core::{
        add::Add, amp::Amp, ring_buffer_record::RingBufferRecord, ring_buffer_playback::RingBufferPlayback, constant::Constant,
        placeholder::Placeholder, proc_once_share::ProcOnceShare, Node,
    },
    signal::C2f64,
};

pub fn delay_fx<A: Node<C2f64> + 'static>(
    node: impl AsMut<A> + 'static,
    sample_rate: usize,
    delay: f64,
    feedback: f64,
) -> ProcOnceShare<
    C2f64,
    RingBufferRecord<
        C2f64,
        Placeholder<C2f64, dyn Node<C2f64>, Box<dyn Node<C2f64>>>,
        Placeholder<C2f64, dyn Node<C2f64>, Box<dyn Node<C2f64>>>,
    >,
    RingBufferRecord<
        C2f64,
        Placeholder<C2f64, dyn Node<C2f64>, Box<dyn Node<C2f64>>>,
        Placeholder<C2f64, dyn Node<C2f64>, Box<dyn Node<C2f64>>>,
    >,
> {
    let mut p = Placeholder::new(None);
    let mut ps = p.setter();
    let buffer = ProcOnceShare::new(RingBufferRecord::new(p, sample_rate));
    unsafe {
        ps.set(Box::new(Add::new(
            node,
            Amp::new(
                RingBufferPlayback::new(Constant::from(delay), buffer.clone()),
                Constant::from(C2f64([feedback, feedback])),
            ),
        )) as Box<dyn Node<C2f64>>);
    }
    buffer
}
