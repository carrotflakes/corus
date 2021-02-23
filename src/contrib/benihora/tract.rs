use std::f64::consts::PI;

use crate::contrib::rand::Rand;

use super::{F, glottis::Glottis};

pub struct Tract {
    n: usize,
    blade_start: usize,
    tip_start: usize,
    lip_start: usize,

    r: Vec<F>,
    l: Vec<F>,
    junction_output_r: Vec<F>,
    junction_output_l: Vec<F>,
    reflection: Vec<F>,
    new_reflection: Vec<F>,
    max_amplitude: Vec<F>, // 表示時用

    diameter: Vec<F>,
    rest_diameter: Vec<F>,
    target_diameter: Vec<F>,

    a: Vec<F>,

    glottal_reflection: F,
    lip_reflection: F,
    last_obstruction: usize,
    fade: F,           // 0.9999
    movement_speed: F, // CM per second

    transients: Vec<Transient>,

    lip_output: F,
    nose_output: F,

    velum_target: F,

    nose_length: usize,
    nose_start: usize,
    nose_r: Vec<F>,
    nose_l: Vec<F>,
    nose_junction_output_r: Vec<F>,
    nose_junction_output_l: Vec<F>,
    nose_reflection: Vec<F>,
    nose_diameter: Vec<F>,
    nose_a: Vec<F>,
    nose_max_amplitude: Vec<F>, // 表示時用

    reflection_left: F,
    reflection_right: F,
    reflection_nose: F,
    new_reflection_left: F,
    new_reflection_right: F,
    new_reflection_nose: F,

    rand: Rand,
}

struct Transient {
    position: usize,
    time_alive: F,
    life_time: F,
    strength: F,
    exponent: F,
}

impl Tract {
    pub fn new() -> Self {
        let n = 44usize;
        let diameter: Vec<_> = (0..n)
            .map(|i| {
                if i < (7.0 * n as F / 44.0 - 0.5) as usize {
                    0.6
                } else if i < (12.0 * n as F / 44.0) as usize {
                    1.1
                } else {
                    1.5
                }
            })
            .collect();

        let nose_length = (28.0 * n as F / 44.0).floor() as usize;
        let mut tract = Tract {
            n,
            blade_start: (10.0 * n as f32 / 44.0).floor() as usize,
            tip_start: (32.0 * n as f32 / 44.0).floor() as usize,
            lip_start: (39.0 * n as f32 / 44.0).floor() as usize,
            r: vec![0.0; n],
            l: vec![0.0; n],
            reflection: vec![0.0; n + 1],
            new_reflection: vec![0.0; n + 1],
            junction_output_r: vec![0.0; n + 1],
            junction_output_l: vec![0.0; n + 1],
            max_amplitude: vec![0.0; n],
            diameter: diameter.clone(),
            rest_diameter: diameter.clone(),
            target_diameter: diameter.clone(),
            a: vec![0.0; n],
            glottal_reflection: 0.75,
            lip_reflection: -0.85,
            last_obstruction: usize::MAX,
            fade: 0.999,
            movement_speed: 15.0,
            transients: Vec::new(),
            lip_output: 0.0,
            nose_output: 0.0,
            velum_target: 0.5, // 0.01

            nose_length,
            nose_start: n - nose_length + 1,
            nose_r: vec![0.0; nose_length],
            nose_l: vec![0.0; nose_length],
            nose_junction_output_r: vec![0.0; nose_length + 1],
            nose_junction_output_l: vec![0.0; nose_length + 1],
            nose_reflection: vec![0.0; nose_length + 1],
            nose_diameter: (0..nose_length)
                .map(|i| {
                    let d = 2.0 * i as F / nose_length as F;
                    (1.9 as F).min(if d < 1.0 {
                        0.4 + 1.6 * d
                    } else {
                        0.5 + 1.5 * (2.0 - d)
                    })
                })
                .collect(),
            nose_a: vec![0.0; nose_length],
            nose_max_amplitude: vec![0.0; nose_length],

            reflection_left: 0.0,
            reflection_right: 0.0,
            reflection_nose: 0.0,
            new_reflection_left: 0.0,
            new_reflection_right: 0.0,
            new_reflection_nose: 0.0,

            rand: Rand::new(0),
        };

        tract.calculate_reflections();
        tract.calculate_nose_reflections();
        tract.nose_diameter[0] = tract.velum_target;
        // dbg!(&tract.nose_diameter);

        tract.set_rest_diameter();//
        // dbg!(&tract.rest_diameter);
        tract
    }

    pub fn run_step(
        &mut self,
        glottal_output: F,
        turbulence_noise: F,
        lambda: F,
        sample_rate: usize,
        glottis: &mut Glottis,
    ) -> F {
        let update_amplitudes = self.rand.next_f32() < 0.1;

        // mouth
        self.process_transients(sample_rate);
        self.add_turbulence_noise(turbulence_noise, glottis);

        //self.glottalReflection = -0.8 + 1.6 * Glottis.newTenseness;
        self.junction_output_r[0] = self.l[0] * self.glottal_reflection + glottal_output;
        self.junction_output_l[self.n] = self.r[self.n - 1] * self.lip_reflection;

        for i in 1..self.n {
            let r = self.reflection[i] * (1.0 - lambda) + self.new_reflection[i] * lambda;
            let w = r * (self.r[i - 1] + self.l[i]);
            self.junction_output_r[i] = self.r[i - 1] - w;
            self.junction_output_l[i] = self.l[i] + w;
        }

        //now at junction with nose
        let i = self.nose_start;
        let r = self.new_reflection_left * (1.0 - lambda) + self.reflection_left * lambda;
        self.junction_output_l[i] = r * self.r[i - 1] + (1.0 + r) * (self.nose_l[0] + self.l[i]);
        let r = self.new_reflection_right * (1.0 - lambda) + self.reflection_right * lambda;
        self.junction_output_r[i] = r * self.l[i] + (1.0 + r) * (self.r[i - 1] + self.nose_l[0]);
        let r = self.new_reflection_nose * (1.0 - lambda) + self.reflection_nose * lambda;
        self.nose_junction_output_r[0] =
            r * self.nose_l[0] + (1.0 + r) * (self.l[i] + self.r[i - 1]);

        for i in 0..self.n {
            self.r[i] = (self.junction_output_r[i] * 0.999).clamp(-1.0, 1.0);
            self.l[i] = (self.junction_output_l[i + 1] * 0.999).clamp(-1.0, 1.0);

            if update_amplitudes {
                let amplitude = (self.r[i] + self.l[i]).abs();
                self.max_amplitude[i] = if amplitude > self.max_amplitude[i] {
                    amplitude
                } else {
                    self.max_amplitude[i] * 0.999
                };
            }
        }

        self.lip_output = self.r[self.n - 1];

        // nose
        self.nose_junction_output_l[self.nose_length] =
            self.nose_r[self.nose_length - 1] * self.lip_reflection;

        for i in 1..self.nose_length {
            let w = self.nose_reflection[i] * (self.nose_r[i - 1] + self.nose_l[i]);
            self.nose_junction_output_r[i] = self.nose_r[i - 1] - w;
            self.nose_junction_output_l[i] = self.nose_l[i] + w;
        }

        for i in 0..self.nose_length {
            self.nose_r[i] = (self.nose_junction_output_r[i] * self.fade).clamp(-1.0, 1.0);
            self.nose_l[i] = (self.nose_junction_output_l[i + 1] * self.fade).clamp(-1.0, 1.0);

            if update_amplitudes {
                let amplitude = (self.nose_r[i] + self.nose_l[i]).abs();
                self.nose_max_amplitude[i] = if amplitude > self.nose_max_amplitude[i] {
                    amplitude
                } else {
                    self.nose_max_amplitude[i] * 0.999
                };
            }
        }

        self.nose_output = self.nose_r[self.nose_length - 1];

        self.lip_output + self.nose_output
    }

    pub fn finish_block(&mut self, block_time: F) {
        self.reshape_tract(block_time);
        self.calculate_reflections()
    }

    fn reshape_tract(&mut self, delta_time: F) {
        // mouth
        let amount = delta_time * self.movement_speed;
        let mut new_last_obstruction = usize::MAX; // 閉塞フラグ
        for i in 0..self.n {
            if self.diameter[i] <= 0.0 {
                new_last_obstruction = i;
            }
            let slow_return = if i < self.nose_start {
                0.6
            } else if i >= self.tip_start {
                1.0
            } else {
                0.6 + 0.4 * (i as F - self.nose_start as F)
                    / (self.tip_start as F - self.nose_start as F)
            };
            self.diameter[i] = move_towards(
                self.diameter[i],
                self.target_diameter[i],
                slow_return * amount,
                2.0 * amount,
            );
        }
        if self.last_obstruction != usize::MAX
            && new_last_obstruction == usize::MAX
            && self.nose_a[0] < 0.05
        {
            self.add_transient(self.last_obstruction);
        }
        self.last_obstruction = new_last_obstruction;

        // nose
        let amount = delta_time * self.movement_speed;
        self.nose_diameter[0] = move_towards(
            self.nose_diameter[0],
            self.velum_target,
            amount * 0.25,
            amount * 0.1,
        );
        self.nose_a[0] = self.nose_diameter[0].powi(2);
    }

    fn calculate_reflections(&mut self) {
        for i in 0..self.n {
            self.a[i] = self.diameter[i] * self.diameter[i]; //ignoring PI etc.
        }
        for i in 1..self.n {
            self.reflection[i] = self.new_reflection[i];
            self.new_reflection[i] = if self.a[i] == 0.0 {
                0.999
            }
            //to prevent some bad behaviour if 0
            else {
                (self.a[i - 1] - self.a[i]) / (self.a[i - 1] + self.a[i])
            };
        }

        //now at junction with nose

        self.reflection_left = self.new_reflection_left;
        self.reflection_right = self.new_reflection_right;
        self.reflection_nose = self.new_reflection_nose;
        let sum = self.a[self.nose_start] + self.a[self.nose_start + 1] + self.nose_a[0];
        self.new_reflection_left = (2.0 * self.a[self.nose_start] - sum) / sum;
        self.new_reflection_right = (2.0 * self.a[self.nose_start + 1] - sum) / sum;
        self.new_reflection_nose = (2.0 * self.nose_a[0] - sum) / sum;
    }

    fn calculate_nose_reflections(&mut self) {
        for i in 0..self.nose_length {
            self.nose_a[i] = self.nose_diameter[i].powi(2);
        }
        for i in 1..self.nose_length {
            self.nose_reflection[i] =
                (self.nose_a[i - 1] - self.nose_a[i]) / (self.nose_a[i - 1] + self.nose_a[i]);
        }
    }

    fn add_transient(&mut self, position: usize) {
        self.transients.push(Transient {
            position,
            time_alive: 0.0,
            life_time: 0.2,
            strength: 0.3,
            exponent: 200.0,
        });
    }

    fn process_transients(&mut self, sample_rate: usize) {
        for trans in self.transients.iter_mut() {
            let amplitude = trans.strength * (2 as F).powf(-trans.exponent * trans.time_alive);
            self.r[trans.position] += amplitude / 2.0;
            self.l[trans.position] += amplitude / 2.0;
            trans.time_alive += 1.0 / (sample_rate as F * 2.0);
        }
        self.transients.retain(|t| t.life_time <= t.time_alive)
    }

    fn add_turbulence_noise(&mut self, turbulence_noise: F, glottis: &mut Glottis) {
        // for touch in touchesWithMouse {
        //     if (touch.index<2 || touch.index>Tract.n) continue;
        //     if (touch.diameter<=0) continue;
        //     let intensity = touch.fricative_intensity;
        //     if (intensity == 0) continue;
        //     this.addTurbulenceNoiseAtIndex(0.66*turbulence_noise*intensity, touch.index, touch.diameter);
        // }
    }

    fn add_turbulence_noise_at_index(
        &mut self,
        turbulence_noise: F,
        index: F,
        diameter: F,
        glottis: &mut Glottis,
    ) {
        let i = index.floor() as usize;
        let delta = index - i as F;
        let turbulence_noise = turbulence_noise * glottis.get_noise_modulator();

        let thinness0 = (8.0 * (0.7 - diameter)).clamp(0.0, 1.0);
        let openness = (30.0 * (diameter - 0.3)).clamp(0.0, 1.0);
        let noise0 = turbulence_noise * (1.0 - delta) * thinness0 * openness;
        let noise1 = turbulence_noise * delta * thinness0 * openness;
        self.r[i + 1] += noise0 / 2.0;
        self.l[i + 1] += noise0 / 2.0;
        self.r[i + 2] += noise1 / 2.0;
        self.l[i + 2] += noise1 / 2.0;
    }

    pub fn set_rest_diameter(&mut self) {
        let tongue_index = 12.9;
        let tongue_diameter = 2.43;
        let grid_offset = 1.7;

        for i in self.blade_start..self.lip_start {
            let t = 1.1 * PI * (tongue_index - i as F) / (self.tip_start - self.blade_start) as F;
            let fixed_tongue_diameter = 2.0 + (tongue_diameter - 2.0) / 1.5;
            let mut curve = (1.5 - fixed_tongue_diameter + grid_offset) * t.cos();
            if i == self.blade_start - 2 || i == self.lip_start - 1 {
                curve *= 0.8;
            }
            if i == self.blade_start || i == self.lip_start - 2 {
                curve *= 0.94;
            }
            self.rest_diameter[i] = 1.5 - curve;
            self.target_diameter[i] = self.rest_diameter[i];//
            self.diameter[i] = self.rest_diameter[i];//
        }
        // dbg!(&self.rest_diameter);
    }
}

fn move_towards(current: F, target: F, up: F, down: F) -> F {
    if current < target {
        target.min(current + up)
    } else {
        target.max(current - down)
    }
}
