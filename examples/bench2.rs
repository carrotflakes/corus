mod write_to_file;

use corus::{EventQueue, core::{Node, add::Add, amp::Amp, var::Var, controllable::Controllable, mix::Mix, param::Param, placeholder::Placeholder, share::Share, ring_buffer_playback::RingBufferPlayback, ring_buffer_record::RingBufferRecord, sine::Sine}, signal::C1f64};

const SAMPLE_RATE: usize = 48000;

fn f1() {
    let mut nodes = Vec::new();
    let modulator = Share::new(Amp::new(
        Sine::new(Var::from(4.0)),
        Var::from(20.0),
    ));
    for i in 0..100 {
        let freq = Controllable::new(Param::new());
        let mut freq_ctrl = freq.controller();
        let f = 440.0 * (i + 1) as f64;
        freq_ctrl.lock().set_value_at_time(0.0, f);
        freq_ctrl.lock().linear_ramp_to_value_at_time(1.0, f * 2.0);
        freq_ctrl.lock().exponential_ramp_to_value_at_time(2.0, f);
        freq_ctrl.lock().set_target_at_time(3.0, 0.01, 1.0);
        nodes.push(Box::new(Amp::new(
            Sine::new(Add::new(freq, modulator.clone())),
            Var::from(1.0 / (i + 1) as f64),
        )) as Box<dyn Node<Output = C1f64>>);
    }

    let mix = Mix::new(nodes);

    let mix = {
        let mut p = Placeholder::new(None);
        let mut ps = p.setter();
        let buffer = Share::new(RingBufferRecord::new(p, SAMPLE_RATE));
        unsafe {
            ps.set(Box::new(Add::new(
                mix,
                Amp::new(
                    RingBufferPlayback::new(Var::from(0.5), buffer.clone()),
                    Var::from(0.5),
                ),
            )) as Box<dyn Node<Output = C1f64>>);
        }
        buffer
    };

    let node = Amp::new(mix, Var::from(0.1));

    write_to_file::write_to_file("bench2_1.wav", SAMPLE_RATE, 5.0, node, None, None);
}

// fn f2() {
//     use corus::core::param2::ParamEventScheduleNode;
//     let mut event_queue = EventQueue::new();
//     let mut nodes = Vec::new();
//     let modulator = Share::new(Amp::new(
//         Sine::new(Var::from(4.0)),
//         Var::from(20.0),
//     ));
//     for i in 0..100 {
//         let mut freq = ParamEventScheduleNode::new(&mut event_queue);
//         let freq_ctrl = freq.get_scheduler();
//         let f = 440.0 * (i + 1) as f64;
//         freq_ctrl.lock().unwrap().set_value_at_time(0.0, f);
//         freq_ctrl.lock().unwrap().linear_ramp_to_value_at_time(1.0, f * 2.0);
//         freq_ctrl.lock().unwrap().exponential_ramp_to_value_at_time(2.0, f);
//         freq_ctrl.lock().unwrap().set_target_at_time(3.0, 0.01, 1.0);
//         nodes.push(Box::new(Amp::new(
//             Sine::new(Add::new(freq, modulator.clone())),
//             Var::from(1.0 / (i + 1) as f64),
//         )) as Box<dyn Node<C1f64>>);
//     }

//     let mix = Mix::new(nodes);

//     let mix = {
//         let mut p = Placeholder::new(None);
//         let mut ps = p.setter();
//         let buffer = Share::new(RingBufferRecord::new(p, SAMPLE_RATE));
//         unsafe {
//             ps.set(Box::new(Add::new(
//                 mix,
//                 Amp::new(
//                     RingBufferPlayback::new(Var::from(0.5), buffer.clone()),
//                     Var::from(0.5),
//                 ),
//             )) as Box<dyn Node<C1f64>>);
//         }
//         buffer
//     };

//     let node = Amp::new(mix, Var::from(0.1));

//     let node = event_queue.dispatch_node(node);

//     write_to_file::write_to_file("bench2_2.wav", SAMPLE_RATE, 5.0, node, None, None);
// }

fn f2() {
    use corus::core::param3::{ParamEventScheduleNode};
    let mut nodes = Vec::new();
    let modulator = Share::new(Amp::new(
        Sine::new(Var::from(4.0)),
        Var::from(20.0),
    ));
    for i in 0..100 {
        let mut freq = ParamEventScheduleNode::new();
        let freq_ctrl = freq.get_scheduler();
        let f = 440.0 * (i + 1) as f64;
        freq_ctrl.lock().unwrap().set_value_at_time(0.0, f);
        freq_ctrl.lock().unwrap().linear_ramp_to_value_at_time(1.0, f * 2.0);
        freq_ctrl.lock().unwrap().exponential_ramp_to_value_at_time(2.0, f);
        freq_ctrl.lock().unwrap().set_target_at_time(3.0, 0.01, 1.0);
        nodes.push(Box::new(Amp::new(
            Sine::new(Add::new(freq, modulator.clone())),
            Var::from(1.0 / (i + 1) as f64),
        )) as Box<dyn Node<Output = C1f64>>);
    }

    let mix = Mix::new(nodes);

    let mix = {
        let mut p = Placeholder::new(None);
        let mut ps = p.setter();
        let buffer = Share::new(RingBufferRecord::new(p, SAMPLE_RATE));
        unsafe {
            ps.set(Box::new(Add::new(
                mix,
                Amp::new(
                    RingBufferPlayback::new(Var::from(0.5), buffer.clone()),
                    Var::from(0.5),
                ),
            )) as Box<dyn Node<Output = C1f64>>);
        }
        buffer
    };

    let node = Amp::new(mix, Var::from(0.1));

    write_to_file::write_to_file("bench2_2.wav", SAMPLE_RATE, 5.0, node, None, None);
}

fn main() {
    f1();
    f2();
}
