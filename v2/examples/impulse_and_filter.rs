use corus_v2::{
    nodes::{effects::SchroederReverb, impulse::Impulse},
    signal::IntoStereo,
    unsafe_wrapper::UnsafeWrapper,
    EventQueue, ProccessContext,
};

fn main() {
    let mut ctx = ProccessContext::new(44100.0);
    let mut event_queue = EventQueue::new();
    let mut impulse = UnsafeWrapper::new(Impulse::new());

    event_queue.push(0.0, impulse.make_event(|impulse, _| impulse.set(1.0)));

    let mut reverb = SchroederReverb::new(44100);
    // let mut filter = corus_v2::nodes::comb_filter::CombFilter::new(44100);
    // let mut filter = corus_v2::nodes::all_pass_filter::AllPassFilter::new(44100);
    // let mut er = corus_v2::nodes::effects::EarlyReflections::new();

    let name = "impulse_and_filter.wav";
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: ctx.sample_rate().round() as u32,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(name, spec).unwrap();
    for _ in 0..44100 * 3 {
        event_queue.dispatch(ctx.current_time());

        let x = impulse.process(&ctx);
        let x = reverb.process(&ctx, x);
        // let x = filter.process(&ctx, x, 0.25, 0.5);
        // let x = er.process(&ctx, x.into_stereo_with_pan(0.0));
        let [l, r] = x.into_stereo_with_pan(0.0);
        writer
            .write_sample((l * std::i16::MAX as f64) as i16)
            .unwrap();
        writer
            .write_sample((r * std::i16::MAX as f64) as i16)
            .unwrap();

        ctx.next();
    }
    writer.finalize().unwrap();
}
