use crate::{core::constant::Constant, signal::{C1f64, Mono}};

use super::{envelope2::AdsrEnvelope, fm_synth::FmSynth, rand::Rand};

pub fn rand_fm_synth(
    seed: u32,
) -> FmSynth<Constant<C1f64>, Constant<C1f64>> {
    let mut rand = Rand::new(seed);
    let f = |rand: &mut Rand, root: bool, inputs: [f64; 4]| {
        let rate_base = if root {
            1.0
        } else {
            8.0 / 2.0f64.powi(rand.next_u32() as i32 % 6)
        };
        let d = if root {
            rand.next_f64().powi(2) * 0.8 + 0.2
        } else {
            rand.next_f64() * 0.95 + 0.05
        };
        let adsr = AdsrEnvelope::new(
            rand.next_f64().powi(4),
            d,
            (1.0 + rand.next_f64().powi(2) * 2.0) / d, //rand.next_f64().powi(2),
            rand.next_f64().powi(2) * 2.0,
        );
        (
            Constant::new(C1f64::from_m(rate_base + (rand.next_f64() - 0.5) * 0.05)),
            Constant::new(C1f64::from_m((rand.next_f64() - 0.5) * 0.05)),
            adsr,
            inputs,
        )
    };
    let amps = [
        rand.next_f64().powi(2) * 5000.0,
        rand.next_f64() * 5000.0,
        rand.next_f64().powi(3) * 100.0,
    ];
    FmSynth::new([
        f(&mut rand, false, [0.0, 0.0, 0.0, 0.0]),
        f(&mut rand, false, [amps[0], 0.0, 0.0, 0.0]),
        f(&mut rand, false, [0.0, 0.0, 0.0, 0.0]),
        f(&mut rand, true, [0.0, amps[1], amps[2], 0.0]),
    ],
    [0.0, 0.0, 0.0, 1.0],
    )
}
