fn main() {
    let sample_rate = 96000;

    let mut rng: rand::rngs::StdRng = rand::SeedableRng::from_seed([0; 32]);

    let dir = "./generated";
    let _ = std::fs::create_dir(dir);

    for i in 0..100 {
        let wt = rand_wt::Config {
            least_depth: 2,
            variable_num: 0,
        }
        .generate(&mut rng);
        let wt = wt.build();

        let name = format!("{}/{:>02}.wav", dir, i);
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(name, spec).unwrap();
        let n = sample_rate / 440;
        for i in 0..n {
            let x = wt(i as f64 / n as f64);
            writer
                .write_sample((x * std::i16::MAX as f64) as i16)
                .unwrap();
        }
        writer.finalize().unwrap();
    }
}
