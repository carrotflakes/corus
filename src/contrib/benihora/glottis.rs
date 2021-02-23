use std::f64::consts::PI;

use super::{F, simplex1};

pub struct Glottis {
    old_frequency: F,
    new_frequency: F,
    ui_frequency: F,
    smooth_frequency: F,

    old_tenseness: F,
    new_tenseness: F,
    ui_tenseness: F,

    vibrato_amount: F,
    vibrato_frequency: F,

    total_time: F,
    time_in_waveform: F,
    waveform_length: F,

    intensity: F,
    loudness: F,

    is_touched: bool,
    is_touching_somewhere: bool,
    //ctx: backCtx,
    // touch: F,
    // x: F,
    // y: F,

    alpha: F,
    e0: F,
    epsilon: F,
    shift: F,
    delta: F,
    te: F,
    omega: F,

    auto_wobble: bool,
}

impl Glottis {
    pub fn new() -> Self {
        let mut glottis = Self {
            old_frequency: 140.0,
            new_frequency: 140.0,
            ui_frequency: 140.0,
            smooth_frequency: 140.0,

            old_tenseness: 0.6,
            new_tenseness: 0.6,
            ui_tenseness: 0.8, // 0.6

            vibrato_amount: 0.005,
            vibrato_frequency: 6.0,

            total_time: 0.0,
            intensity: 0.0,
            loudness: 1.0,
            is_touched: false,
            is_touching_somewhere: false,

            time_in_waveform: 0.0,
            waveform_length: 0.0,

            alpha: 0.0,
            e0: 0.0,
            epsilon: 0.0,
            shift: 0.0,
            delta: 0.0,
            te: 0.0,
            omega: 0.0,

            auto_wobble: false,
        };
        glottis.setup_waveform(0.0);
        glottis
    }

    pub fn run_step(&mut self, sample_rate: usize, lambda: F, input: F) -> F {
        self.ui_tenseness = (self.total_time * 2.0).sin() * 0.5 + 0.5;
        self.loudness = self.ui_tenseness.powf(0.25);

        let time_step = 1.0 / sample_rate as F;
        self.time_in_waveform += time_step;
        self.total_time += time_step;
        if self.waveform_length < self.time_in_waveform {
            self.time_in_waveform -= self.waveform_length;
            self.setup_waveform(lambda);
        }
        let out = self.normalized_lf_waveform(self.time_in_waveform / self.waveform_length);
        let aspiration = self.intensity
            * (1.0 - self.ui_tenseness.sqrt())
            * self.get_noise_modulator()
            * input
            * (0.2 + 0.02 * simplex1(self.total_time * 1.99));
        out + aspiration
    }

    pub fn get_noise_modulator(&mut self) -> F {
        let voiced =
            0.1 + 0.2 * 0.0f64.max((PI * 2.0 * self.time_in_waveform / self.waveform_length).sin());
        self.ui_tenseness * self.intensity * voiced
            + (1.0 - self.ui_tenseness * self.intensity) * 0.3
    }

    pub fn update_block(&mut self, block_time: F) {
        let always_voice = true;

        let mut vibrato = self.vibrato_amount * (2.0 * PI * self.total_time * self.vibrato_frequency).sin();
        vibrato += 0.02 * simplex1(self.total_time * 4.07);
        vibrato += 0.04 * simplex1(self.total_time * 2.15);
        if self.auto_wobble {
            vibrato += 0.2 * simplex1(self.total_time * 0.98);
            vibrato += 0.4 * simplex1(self.total_time * 0.5);
        }

        if self.ui_frequency > self.smooth_frequency {
            self.smooth_frequency = (self.smooth_frequency * 1.1).min(self.ui_frequency)
        }
        if self.ui_frequency < self.smooth_frequency {
            self.smooth_frequency = (self.smooth_frequency / 1.1).max(self.ui_frequency)
        }
        self.old_frequency = self.new_frequency;
        self.new_frequency = self.smooth_frequency * (1.0 + vibrato);

        self.old_tenseness = self.new_tenseness;
        self.new_tenseness = self.ui_tenseness
            + 0.1 * simplex1(self.total_time * 0.46)
            + 0.05 * simplex1(self.total_time * 0.36);

        if !self.is_touched && (always_voice || self.is_touching_somewhere) {
            self.new_tenseness += (3.0 - self.ui_tenseness) * (1.0 - self.intensity)
        }

        if self.is_touched || always_voice || self.is_touching_somewhere {
            self.intensity += 0.13
        } else {
            self.intensity -= block_time * 5.0
        }
        self.intensity = self.intensity.clamp(0.0, 1.0);
    }

    fn setup_waveform(&mut self, lambda: F) {
        let frequency = self.old_frequency * (1.0 - lambda) + self.new_frequency * lambda;
        let tenseness = self.old_tenseness * (1.0 - lambda) + self.new_tenseness * lambda;
        let rd = (3.0 * (1.0 - tenseness)).clamp(0.5, 2.7);
        self.waveform_length = 1.0 / frequency;

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
        self.alpha = alpha;
        self.e0 = e0;
        self.epsilon = epsilon;
        self.shift = shift;
        self.delta = delta;
        self.te = te;
        self.omega = omega;
    }

    fn normalized_lf_waveform(&self, t: F) -> F {
        self.intensity
            * self.loudness
            * if self.te < t {
                (-(-self.epsilon * (t - self.te)).exp() + self.shift) / self.delta
            } else {
                self.e0 * (self.alpha * t).exp() * (self.omega * t).sin()
            }
    }
}
