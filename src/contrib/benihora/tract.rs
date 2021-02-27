use std::f64::consts::PI;

use crate::contrib::rand::Rand;

use super::F;
const NOSE_OFFSET: F = 0.8;

pub struct Tract {
    pub mouth: Mouth,
    pub nose: Nose,
    pub movement_speed: F, // CM per second
    rand: Rand,            // for update max_amplitude
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
        let n = 44;
        let mouth_length = n;
        let nose_length = (28.0 * mouth_length as F / 44.0).floor() as usize; // magic number 44???
        let mut tract = Tract {
            mouth: Mouth::new(mouth_length),
            nose: Nose::new(nose_length, mouth_length),
            movement_speed: 15.0,
            rand: Rand::new(0),
        };

        tract.mouth.calculate_reflections(&tract.nose);
        tract.nose.calculate_reflections();
        tract.nose.diameter[0] = tract.nose.velum_target;
        tract.calculate_diameter();

        tract
    }

    pub fn run_step(
        &mut self,
        time: f64,
        glottal_output: F,
        turbulence_noise: F,
        lambda: F,
        sample_rate: usize,
    ) -> F {
        let update_amplitudes = self.rand.next_f32() < 0.1;

        let lip_output = self.mouth.run_step(
            &mut self.nose,
            time,
            glottal_output,
            turbulence_noise,
            lambda,
            sample_rate,
            update_amplitudes,
        );
        let nose_out = self.nose.run_step(&self.mouth, update_amplitudes);

        lip_output + nose_out
    }

    pub fn update_block(&mut self, block_time: F) {
        self.mouth
            .reshape(block_time * self.movement_speed, &self.nose);
        self.nose.reshape(block_time * self.movement_speed);
        self.mouth.calculate_reflections(&self.nose);
    }

    pub fn calculate_diameter(&mut self) {
        self.mouth.calculate_diameter();

        let open_velum = self
            .mouth
            .other_constrictions
            .iter()
            .find(|constriction| {
                constriction.index > self.nose.start as F
                    && constriction.diameter < -NOSE_OFFSET
                    && constriction.end_time.is_none()
            })
            .is_some();
        self.nose.velum_target = if open_velum { 0.4 } else { 0.01 };
    }
}

pub struct Mouth {
    length: usize,
    blade_start: usize,
    tip_start: usize,
    lip_start: usize,

    diameter: Vec<F>,
    rest_diameter: Vec<F>,
    target_diameter: Vec<F>,

    a: Vec<F>,

    reflection_left: F,
    reflection_right: F,
    reflection_nose: F,
    new_reflection_left: F,
    new_reflection_right: F,
    new_reflection_nose: F,

    r: Vec<F>,
    l: Vec<F>,
    junction_output_r: Vec<F>,
    junction_output_l: Vec<F>,
    reflection: Vec<F>,
    new_reflection: Vec<F>,

    max_amplitude: Vec<F>, // 表示時用

    glottal_reflection: F,
    lip_reflection: F,

    last_obstruction: usize,

    transients: Vec<Transient>,
    pub tongue: (F, F), // (index, diameter) // TODO index -> rate
    pub other_constrictions: Vec<Constriction>,
}

impl Mouth {
    fn new(length: usize) -> Self {
        let diameter: Vec<_> = (0..length)
            .map(|i| {
                if (i as f64) < (7.0 * length as F / 44.0 - 0.5) {
                    0.6
                } else if (i as f64) < (12.0 * length as F / 44.0) {
                    1.1
                } else {
                    1.5
                }
            })
            .collect();
        Mouth {
            length,
            blade_start: (10.0 * length as f32 / 44.0).floor() as usize,
            tip_start: (32.0 * length as f32 / 44.0).floor() as usize,
            lip_start: (39.0 * length as f32 / 44.0).floor() as usize,
            r: vec![0.0; length],
            l: vec![0.0; length],
            reflection: vec![0.0; length + 1],
            new_reflection: vec![0.0; length + 1],
            junction_output_r: vec![0.0; length + 1],
            junction_output_l: vec![0.0; length + 1],
            max_amplitude: vec![0.0; length],
            diameter: diameter.clone(),
            rest_diameter: diameter.clone(),
            target_diameter: diameter.clone(),
            a: vec![0.0; length],
            glottal_reflection: 0.75,
            lip_reflection: -0.85,
            last_obstruction: usize::MAX,
            transients: Vec::new(),

            reflection_left: 0.0,
            reflection_right: 0.0,
            reflection_nose: 0.0,
            new_reflection_left: 0.0,
            new_reflection_right: 0.0,
            new_reflection_nose: 0.0,
            tongue: (12.9, 2.43),
            other_constrictions: Vec::new(),
        }
    }

    fn run_step(
        &mut self,
        nose: &mut Nose,
        time: f64,
        glottal_output: F,
        turbulence_noise: F,
        lambda: F,
        sample_rate: usize,
        update_amplitudes: bool,
    ) -> F {
        // mouth
        self.process_transients(sample_rate);
        self.add_turbulence_noise(time, turbulence_noise);

        //self.glottalReflection = -0.8 + 1.6 * Glottis.newTenseness;
        self.junction_output_r[0] = self.l[0] * self.glottal_reflection + glottal_output;
        self.junction_output_l[self.length] = self.r[self.length - 1] * self.lip_reflection;

        for i in 1..self.length {
            let r = self.reflection[i] * (1.0 - lambda) + self.new_reflection[i] * lambda;
            let w = r * (self.r[i - 1] + self.l[i]);
            self.junction_output_r[i] = self.r[i - 1] - w;
            self.junction_output_l[i] = self.l[i] + w;
        }

        //now at junction with nose
        let i = nose.start;
        let r = self.new_reflection_left * (1.0 - lambda) + self.reflection_left * lambda;
        self.junction_output_l[i] = r * self.r[i - 1] + (1.0 + r) * (nose.l[0] + self.l[i]);
        let r = self.new_reflection_right * (1.0 - lambda) + self.reflection_right * lambda;
        self.junction_output_r[i] = r * self.l[i] + (1.0 + r) * (self.r[i - 1] + nose.l[0]);
        let r = self.new_reflection_nose * (1.0 - lambda) + self.reflection_nose * lambda;
        nose.junction_output_r[0] = r * nose.l[0] + (1.0 + r) * (self.l[i] + self.r[i - 1]);

        for i in 0..self.length {
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

        self.r[self.length - 1]
    }

    fn reshape(&mut self, amount: F, nose: &Nose) {
        let mut new_last_obstruction = usize::MAX; // 閉塞フラグ
        for i in 0..self.length {
            if self.diameter[i] <= 0.0 {
                new_last_obstruction = i;
            }
            let slow_return = if i < nose.start {
                0.6
            } else if i >= self.tip_start {
                1.0
            } else {
                0.6 + 0.4 * (i as F - nose.start as F) / (self.tip_start as F - nose.start as F)
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
            && nose.a[0] < 0.05
        {
            self.add_transient(self.last_obstruction);
        }
        self.last_obstruction = new_last_obstruction;
    }

    fn calculate_reflections(&mut self, nose: &Nose) {
        for i in 0..self.length {
            self.a[i] = self.diameter[i].powi(2); //ignoring PI etc.
        }
        for i in 1..self.length {
            self.reflection[i] = self.new_reflection[i];
            self.new_reflection[i] = if self.a[i] == 0.0 {
                // to prevent some bad behaviour if 0
                0.999
            } else {
                (self.a[i - 1] - self.a[i]) / (self.a[i - 1] + self.a[i])
            };
        }

        //now at junction with nose

        self.reflection_left = self.new_reflection_left;
        self.reflection_right = self.new_reflection_right;
        self.reflection_nose = self.new_reflection_nose;
        let sum = self.a[nose.start] + self.a[nose.start + 1] + nose.a[0];
        self.new_reflection_left = (2.0 * self.a[nose.start] - sum) / sum;
        self.new_reflection_right = (2.0 * self.a[nose.start + 1] - sum) / sum;
        self.new_reflection_nose = (2.0 * nose.a[0] - sum) / sum;
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
        self.transients.retain(|t| t.time_alive <= t.life_time)
    }

    fn add_turbulence_noise(&mut self, time: f64, turbulence_noise: F) {
        for constriction in self.other_constrictions.clone() {
            if constriction.index < 2.0 || constriction.index > self.length as F {
                continue;
            }
            if constriction.diameter <= 0.0 {
                continue;
            }
            let intensity = constriction.fricative_intensity(time);
            if intensity == 0.0 {
                continue;
            }
            self.add_turbulence_noise_at_index(
                0.66 * turbulence_noise * intensity,
                constriction.index,
                constriction.diameter,
            );
        }

        // Remove dead constrictions
        self.other_constrictions.retain(|c| {
            c.end_time
                .map(|end_time| time < end_time + 1.0)
                .unwrap_or(true)
        });
    }

    fn add_turbulence_noise_at_index(&mut self, turbulence_noise: F, index: F, diameter: F) {
        let i = index.floor() as usize;
        let delta = index - i as F;

        let thinness0 = (8.0 * (0.7 - diameter)).clamp(0.0, 1.0);
        let openness = (30.0 * (diameter - 0.3)).clamp(0.0, 1.0);
        let noise0 = turbulence_noise * (1.0 - delta) * thinness0 * openness;
        let noise1 = turbulence_noise * delta * thinness0 * openness;
        self.r[i + 1] += noise0 / 2.0;
        self.l[i + 1] += noise0 / 2.0;
        self.r[i + 2] += noise1 / 2.0;
        self.l[i + 2] += noise1 / 2.0;
    }

    pub fn calculate_diameter(&mut self) {
        const GRID_OFFSET: F = 1.7;

        let (tongue_index, tongue_diameter) = self.tongue;

        for i in self.blade_start..self.lip_start {
            let t = 1.1 * PI * (tongue_index - i as F) / (self.tip_start - self.blade_start) as F;
            let fixed_tongue_diameter = 2.0 + (tongue_diameter - 2.0) / 1.5;
            let mut curve = (1.5 - fixed_tongue_diameter + GRID_OFFSET) * t.cos();
            if i == self.blade_start - 2 || i == self.lip_start - 1 {
                curve *= 0.8;
            }
            if i == self.blade_start || i == self.lip_start - 2 {
                curve *= 0.94;
            }
            self.rest_diameter[i] = 1.5 - curve;
        }

        for i in 0..self.length {
            self.target_diameter[i] = self.rest_diameter[i];
        }

        for constriction in self.other_constrictions.iter() {
            let index = constriction.index;
            let mut diameter = constriction.diameter;
            if diameter < -0.85 - NOSE_OFFSET || constriction.end_time.is_some() {
                continue;
            }
            diameter = (diameter - 0.3).max(0.0);

            let width = if index < 25.0 {
                10.0
            } else if index >= self.tip_start as F {
                5.0
            } else {
                10.0 - 5.0 * (index - 25.0) / (self.tip_start as F - 25.0)
            };

            if index >= 2.0 && index < self.length as F && diameter < 3.0 {
                // && y<tractCanvas.height
                let int_index = index.round() as isize;
                for i in -width.ceil() as isize - 1..width as isize + 1 {
                    let idx = int_index + i;

                    if idx < 0 || idx >= self.length as isize {
                        continue;
                    }
                    let idx = idx as usize;
                    let relpos = (idx as F - index).abs() - 0.5;
                    let shrink = if relpos <= 0.0 {
                        0.0
                    } else if relpos > width {
                        1.0
                    } else {
                        0.5 * (1.0 - (PI * relpos / width).cos())
                    };
                    if diameter < self.target_diameter[idx] {
                        self.target_diameter[idx] =
                            diameter + (self.target_diameter[idx] - diameter) * shrink;
                    }
                }
            }
        }
    }

    pub fn tangue_clamp(&self, index: F, diameter: F) -> (F, F) {
        const INNER_RADIUS: F = 2.05;
        const OUTER_RADIUS: F = 3.5;
        let lower_index_bound = self.blade_start as F + 2.0;
        let upper_index_bound = self.tip_start as F - 3.0;
        let index_center = (lower_index_bound + upper_index_bound) * 0.5;

        let mut from_point = (OUTER_RADIUS - diameter) / (OUTER_RADIUS - INNER_RADIUS);
        from_point = from_point.clamp(0.0, 1.0);
        from_point = from_point.powf(0.58) - 0.2 * (from_point.powi(2) - from_point); // horrible kludge to fit curve to straight line
        let diameter = diameter.clamp(INNER_RADIUS, OUTER_RADIUS);
        let out = from_point * 0.5 * (upper_index_bound - lower_index_bound);
        let index = index.clamp(index_center - out, index_center + out);
        (index, diameter)
    }
}

pub struct Nose {
    length: usize,
    start: usize,
    pub diameter: Vec<F>,
    a: Vec<F>,
    r: Vec<F>,
    l: Vec<F>,
    junction_output_r: Vec<F>,
    junction_output_l: Vec<F>,
    reflection: Vec<F>,

    max_amplitude: Vec<F>, // 表示時用

    pub fade: F, // 0.9999
    pub velum_target: F,
}

impl Nose {
    fn new(length: usize, mouth_length: usize) -> Self {
        Nose {
            length,
            start: mouth_length - length + 1,
            r: vec![0.0; length],
            l: vec![0.0; length],
            junction_output_r: vec![0.0; length + 1],
            junction_output_l: vec![0.0; length + 1],
            reflection: vec![0.0; length + 1],
            diameter: (0..length)
                .map(|i| {
                    let d = 2.0 * i as F / length as F;
                    (1.9 as F).min(if d < 1.0 {
                        0.4 + 1.6 * d
                    } else {
                        0.5 + 1.5 * (2.0 - d)
                    })
                })
                .collect(),
            a: vec![0.0; length],
            max_amplitude: vec![0.0; length],
            fade: 0.999,
            velum_target: 0.5, // 0.01
        }
    }

    fn run_step(&mut self, mouth: &Mouth, update_amplitudes: bool) -> F {
        self.junction_output_l[self.length] = self.r[self.length - 1] * mouth.lip_reflection;

        for i in 1..self.length {
            let w = self.reflection[i] * (self.r[i - 1] + self.l[i]);
            self.junction_output_r[i] = self.r[i - 1] - w;
            self.junction_output_l[i] = self.l[i] + w;
        }

        for i in 0..self.length {
            self.r[i] = (self.junction_output_r[i] * self.fade).clamp(-1.0, 1.0);
            self.l[i] = (self.junction_output_l[i + 1] * self.fade).clamp(-1.0, 1.0);

            if update_amplitudes {
                let amplitude = (self.r[i] + self.l[i]).abs();
                self.max_amplitude[i] = if amplitude > self.max_amplitude[i] {
                    amplitude
                } else {
                    self.max_amplitude[i] * 0.999
                };
            }
        }

        self.r[self.length - 1]
    }

    fn reshape(&mut self, amount: F) {
        self.diameter[0] = move_towards(
            self.diameter[0],
            self.velum_target,
            amount * 0.25,
            amount * 0.1,
        );
        self.a[0] = self.diameter[0].powi(2);
    }

    // NOTE: called once only!
    fn calculate_reflections(&mut self) {
        for i in 0..self.length {
            self.a[i] = self.diameter[i].powi(2);
        }
        for i in 1..self.length {
            self.reflection[i] = (self.a[i - 1] - self.a[i]) / (self.a[i - 1] + self.a[i]);
        }
    }
}

fn move_towards(current: F, target: F, up: F, down: F) -> F {
    if current < target {
        target.min(current + up)
    } else {
        target.max(current - down)
    }
}

#[derive(Clone)]
pub struct Constriction {
    pub index: F,
    pub diameter: F,
    pub start_time: f64,
    pub end_time: Option<f64>,
}

impl Constriction {
    fn fricative_intensity(&self, time: f64) -> F {
        let fricative_attack_time = 0.1;
        if let Some(end_time) = self.end_time {
            (1.0 - (time - end_time) / fricative_attack_time).clamp(0.0, 1.0)
        } else {
            ((time - self.start_time) / fricative_attack_time).clamp(0.0, 1.0)
        }
    }
}
