use crate::{
    node::{
        add::Add, amp::Amp, buffer::Buffer, buffer_playback::BufferPlayback, constant::Constant,
        placeholder::Placeholder, proc_once_share::ProcOnceShare, Node,
    },
    signal::C2f32,
};

pub fn delay_fx<A: Node<C2f32> + 'static>(
    node: impl AsMut<A> + 'static,
    sample_rate: usize,
    delay: f32,
    feedback: f32,
) -> ProcOnceShare<
    C2f32,
    Buffer<
        C2f32,
        Placeholder<C2f32, dyn Node<C2f32>, Box<dyn Node<C2f32>>>,
        Placeholder<C2f32, dyn Node<C2f32>, Box<dyn Node<C2f32>>>,
    >,
    Buffer<
        C2f32,
        Placeholder<C2f32, dyn Node<C2f32>, Box<dyn Node<C2f32>>>,
        Placeholder<C2f32, dyn Node<C2f32>, Box<dyn Node<C2f32>>>,
    >,
> {
    let mut p = Placeholder::new();
    let mut ps = p.setter();
    let buffer = ProcOnceShare::new(Buffer::new(p, sample_rate));
    unsafe {
        ps.set(Box::new(Add::new(
            node,
            Amp::new(
                BufferPlayback::new(Constant::from(delay), buffer.clone()),
                Constant::from(C2f32([feedback, feedback])),
            ),
        )) as Box<dyn Node<C2f32>>);
    }
    buffer
}
