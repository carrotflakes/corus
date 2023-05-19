use std::f64::consts::PI;

use crate::lerp;

use super::{simplex1, F};

pub struct Glottis {
    waveform_length: F,
    time_in_waveform: F,
    waveform: Waveform,
}

impl Glottis {
    pub fn new() -> Self {
        Self {
            waveform_length: 1.0 / 140.0,
            time_in_waveform: 0.0,
            waveform: Waveform::new(0.6),
        }
    }

    pub fn compute(
        &mut self,
        time: f64,
        dtime: f64,
        noise: F,
        frequency: F,
        tenseness: F,
        intensity: F,
        loudness: F,
    ) -> F {
        self.time_in_waveform += dtime;
        if self.waveform_length < self.time_in_waveform {
            self.time_in_waveform -= self.waveform_length;
            self.waveform_length = 1.0 / frequency;
            self.waveform = Waveform::new(tenseness)
        }
        let out = intensity
            * loudness
            * self
                .waveform
                .normalized_lf_waveform(self.time_in_waveform / self.waveform_length);
        let noise = self.get_noise_modulator(tenseness * intensity) * noise;
        let aspiration =
            intensity * (1.0 - tenseness.sqrt()) * noise * (0.2 + 0.02 * simplex1(time * 1.99));
        out + aspiration
    }

    fn get_noise_modulator(&mut self, rate: F) -> F {
        let voiced =
            0.1 + 0.2 * 0.0f64.max((PI * 2.0 * self.time_in_waveform / self.waveform_length).sin());
        lerp(0.3, voiced, rate)
    }
}

/// Liljencrants-Fant waveform
struct Waveform {
    alpha: F,
    e0: F,
    epsilon: F,
    shift: F,
    delta: F,
    te: F,
    omega: F,
}

impl Waveform {
    fn new(tenseness: F) -> Self {
        let rd = (3.0 * (1.0 - tenseness)).clamp(0.5, 2.7);

        let ra = -0.01 + 0.048 * rd;
        let rk = 0.224 + 0.118 * rd;
        let rg = (rk / 4.0) * (0.5 + 1.2 * rk) / (0.11 * rd - ra * (0.5 + 1.2 * rk));

        let ta = ra;
        let tp = 1.0 / (2.0 * rg);
        let te = tp + tp * rk;

        let epsilon = 1.0 / ta;
        let shift = (-epsilon * (1.0 - te)).exp();
        let delta = 1.0 - shift; //divide by this to scale RHS

        let rhs_integral = ((1.0 / epsilon) * (shift - 1.0) + (1.0 - te) * shift) / delta;

        let total_lower_integral = -(te - tp) / 2.0 + rhs_integral;
        let total_upper_integral = -total_lower_integral;

        let omega = PI / tp;
        let s = (omega * te).sin();
        // need E0*e^(alpha*Te)*s = -1 (to meet the return at -1)
        // and E0*e^(alpha*Tp/2) * Tp*2/pi = totalUpperIntegral
        //             (our approximation of the integral up to Tp)
        // writing x for e^alpha,
        // have E0*x^Te*s = -1 and E0 * x^(Tp/2) * Tp*2/pi = totalUpperIntegral
        // dividing the second by the first,
        // letting y = x^(Tp/2 - Te),
        // y * Tp*2 / (pi*s) = -totalUpperIntegral;
        let y = -PI * s * total_upper_integral / (tp * 2.0);
        let z = y.ln();
        let alpha = z / (tp / 2.0 - te);
        let e0 = -1.0 / (s * (alpha * te).exp());

        Self {
            alpha,
            e0,
            epsilon,
            shift,
            delta,
            te,
            omega,
        }
    }

    fn normalized_lf_waveform(&self, t: F) -> F {
        if self.te < t {
            (-(-self.epsilon * (t - self.te)).exp() + self.shift) / self.delta
        } else {
            self.e0 * (self.alpha * t).exp() * (self.omega * t).sin()
        }
    }
}
