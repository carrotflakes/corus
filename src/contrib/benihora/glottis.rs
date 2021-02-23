use std::f64::consts::PI;

use super::{simplex1, F};

pub struct Glottis {
    pub frequency: FrequencyCtrl,
    pub tenseness: TensenessCtrl,

    total_time: F,
    time_in_waveform: F,

    waveform: Waveform,

    intensity: F,
    loudness: F,

    is_touched: bool,
    is_touching_somewhere: bool,
}

impl Glottis {
    pub fn new() -> Self {
        Self {
            frequency: FrequencyCtrl::new(140.0, 0.005, 6.0, false),
            tenseness: TensenessCtrl::new(0.6),

            total_time: 0.0,
            intensity: 0.0,
            loudness: 1.0,
            is_touched: true,
            is_touching_somewhere: false,

            time_in_waveform: 0.0,
            waveform: Waveform::new(140.0, 0.6),
        }
    }

    pub fn run_step(&mut self, sample_rate: usize, lambda: F, input: F) -> F {
        // self.tenseness.ui_tenseness = (self.total_time * 2.0).sin() * 0.5 + 0.5;
        // self.loudness = self.tenseness.ui_tenseness.powf(0.25);
        // self.is_touched = 1.0 < self.total_time % 2.0;

        let tenseness = self.tenseness.get(lambda);

        let time_step = 1.0 / sample_rate as F;
        self.time_in_waveform += time_step;
        self.total_time += time_step;
        if self.waveform.length < self.time_in_waveform {
            self.time_in_waveform -= self.waveform.length;
            self.waveform = Waveform::new(self.frequency.get(lambda), tenseness)
        }
        let out = self.intensity
            * self.loudness
            * self
                .waveform
                .normalized_lf_waveform(self.time_in_waveform / self.waveform.length);
        let aspiration = self.intensity
            * (1.0 - tenseness.sqrt())
            * self.get_noise_modulator()
            * input
            * (0.2 + 0.02 * simplex1(self.total_time * 1.99));
        out + aspiration
    }

    pub fn get_noise_modulator(&mut self) -> F {
        let tenseness = self.tenseness.get(0.0); // ?
        let voiced =
            0.1 + 0.2 * 0.0f64.max((PI * 2.0 * self.time_in_waveform / self.waveform.length).sin());
        tenseness * self.intensity * voiced + (1.0 - tenseness * self.intensity) * 0.3
    }

    pub fn update_block(&mut self, block_time: F) {
        let always_voice = false;

        let glottis_open = !self.is_touched && (always_voice || self.is_touching_somewhere);

        self.frequency.update(self.total_time);
        self.tenseness.update(
            self.total_time,
            glottis_open,
            self.intensity,
        );

        if self.is_touched || always_voice || self.is_touching_somewhere {
            self.intensity += 0.13
        } else {
            self.intensity -= block_time * 5.0
        }
        self.intensity = self.intensity.clamp(0.0, 1.0);
    }
}

pub struct FrequencyCtrl {
    old_frequency: F,
    new_frequency: F,
    ui_frequency: F,
    smooth_frequency: F,

    pub vibrato_amount: F,
    pub vibrato_frequency: F,

    pub auto_wobble: bool,
}

impl FrequencyCtrl {
    fn new(frequency: F, vibrato_amount: F, vibrato_frequency: F, auto_wobble: bool) -> Self {
        Self {
            old_frequency: frequency,
            new_frequency: frequency,
            ui_frequency: frequency,
            smooth_frequency: frequency,
            vibrato_amount,
            vibrato_frequency,
            auto_wobble,
        }
    }

    fn update(&mut self, time: F) {
        let mut vibrato = self.vibrato_amount * (2.0 * PI * time * self.vibrato_frequency).sin();
        vibrato += 0.02 * simplex1(time * 4.07);
        vibrato += 0.04 * simplex1(time * 2.15);
        if self.auto_wobble {
            vibrato += 0.2 * simplex1(time * 0.98);
            vibrato += 0.4 * simplex1(time * 0.5);
        }

        if self.ui_frequency > self.smooth_frequency {
            self.smooth_frequency = (self.smooth_frequency * 1.1).min(self.ui_frequency)
        }
        if self.ui_frequency < self.smooth_frequency {
            self.smooth_frequency = (self.smooth_frequency / 1.1).max(self.ui_frequency)
        }
        self.old_frequency = self.new_frequency;
        self.new_frequency = self.smooth_frequency * (1.0 + vibrato);
    }

    fn get(&self, lambda: F) -> F {
        self.old_frequency * (1.0 - lambda) + self.new_frequency * lambda
    }
}
pub struct TensenessCtrl {
    old_tenseness: F,
    new_tenseness: F,
    ui_tenseness: F,
}

impl TensenessCtrl {
    fn new(tenseness: F) -> Self {
        Self {
            old_tenseness: tenseness,
            new_tenseness: tenseness,
            ui_tenseness: tenseness,
        }
    }

    fn update(&mut self, time: F, glottis_open: bool, intensity: F) {
        self.old_tenseness = self.new_tenseness;
        self.new_tenseness =
            self.ui_tenseness + 0.1 * simplex1(time * 0.46) + 0.05 * simplex1(time * 0.36);

        if glottis_open {
            self.new_tenseness += (3.0 - self.ui_tenseness) * (1.0 - intensity)
        }
    }

    fn get(&self, lambda: F) -> F {
        self.old_tenseness * (1.0 - lambda) + self.new_tenseness * lambda
    }
}

struct Waveform {
    length: F,

    alpha: F,
    e0: F,
    epsilon: F,
    shift: F,
    delta: F,
    te: F,
    omega: F,
}

impl Waveform {
    fn new(frequency: F, tenseness: F) -> Self {
        let rd = (3.0 * (1.0 - tenseness)).clamp(0.5, 2.7);
        let waveform_length = 1.0 / frequency;

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
            length: waveform_length,
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
