use std::f64::consts::PI;

use super::{lerp, F};
const NOSE_OFFSET: F = 0.8;

pub struct Tract {
    pub mouth: Mouth,
    pub nose: Nose,
    pub movement_speed: F, // CM per second
}

impl Tract {
    pub fn new() -> Self {
        let mouth_length = 44;
        let nose_length = 28;
        let mut tract = Tract {
            mouth: Mouth::new(mouth_length, nose_length),
            nose: Nose::new(nose_length),
            movement_speed: 15.0,
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
        dtime: F,
    ) -> (F, F) {
        let lip_output = self.mouth.process(
            &mut self.nose,
            time,
            glottal_output,
            turbulence_noise,
            lambda,
            dtime,
        );
        let nose_out = self.nose.run_step();

        (lip_output, nose_out)
    }

    pub fn update_block(&mut self, block_time: F) {
        self.mouth
            .reshape(block_time * self.movement_speed, &self.nose);
        self.nose.reshape(block_time * self.movement_speed);
        self.mouth.calculate_reflections(&self.nose);
    }

    pub fn calculate_diameter(&mut self) {
        self.mouth.calculate_diameter();

        // let velum_open = self.mouth.other_constrictions.iter().any(|constriction| {
        //     constriction.index > self.self.nose_start as F
        //         && constriction.diameter < -NOSE_OFFSET
        //         && constriction.end_time.is_none()
        // });
        // self.nose.velum_target = if velum_open { 0.4 } else { 0.01 };
    }
}

pub struct Mouth {
    length: usize,
    blade_start: usize,
    tip_start: usize,
    lip_start: usize,
    nose_start: usize,

    original_diameter: Vec<F>,
    diameter: Vec<F>,
    target_diameter: Vec<F>,

    area: Vec<F>,

    reflection: Vec<F>,
    new_reflection: Vec<F>,
    reflection_left: F,
    reflection_right: F,
    reflection_nose: F,
    new_reflection_left: F,
    new_reflection_right: F,
    new_reflection_nose: F,

    r: Vec<F>,
    l: Vec<F>,
    r_: Vec<F>,
    l_: Vec<F>,

    glottal_reflection: F,
    lip_reflection: F,

    last_obstruction: usize,

    transients: Vec<Transient>,
    pub tongue: (F, F), // (index, diameter) // TODO index -> rate, should this be here?
    pub other_constrictions: Vec<Constriction>,

    pub fade: F,
}

impl Mouth {
    fn new(length: usize, nose_length: usize) -> Self {
        let diameter: Vec<_> = (0..length)
            .map(|i| {
                if (i as f64) < (7.0 / 44.0 * length as F - 0.5) {
                    0.6
                } else if (i as f64) < (12.0 / 44.0 * length as F) {
                    1.1
                } else {
                    1.5
                }
            })
            .collect();
        Mouth {
            length,
            blade_start: (10.0 / 44.0 * length as f32).floor() as usize,
            tip_start: (32.0 / 44.0 * length as f32).floor() as usize,
            lip_start: (39.0 / 44.0 * length as f32).floor() as usize,
            nose_start: length - nose_length + 1,
            r: vec![0.0; length],
            l: vec![0.0; length],
            r_: vec![0.0; length],
            l_: vec![0.0; length],
            reflection: vec![0.0; length + 1],
            new_reflection: vec![0.0; length + 1],
            diameter: diameter.clone(),
            original_diameter: diameter.clone(),
            target_diameter: diameter.clone(),
            area: vec![0.0; length],
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
            fade: 0.999,
        }
    }

    fn process(
        &mut self,
        nose: &mut Nose,
        time: f64,
        glottal_output: F,
        turbulence_noise: F,
        lambda: F,
        dtime: F,
    ) -> F {
        self.process_transients(dtime);
        self.add_turbulence_noise(time, turbulence_noise);

        //self.glottalReflection = -0.8 + 1.6 * Glottis.newTenseness;
        self.r_[0] = self.l[0] * self.glottal_reflection + glottal_output;
        self.l_[self.length - 1] = self.r[self.length - 1] * self.lip_reflection;

        for i in 1..self.length {
            let r = lerp(self.reflection[i], self.new_reflection[i], lambda);
            let w = r * (self.r[i - 1] + self.l[i]);
            self.r_[i] = self.r[i - 1] - w;
            self.l_[i - 1] = self.l[i] + w;
        }

        //now at junction with nose
        let i = self.nose_start;
        let r = lerp(self.reflection_left, self.new_reflection_left, lambda);
        self.l_[i - 1] = r * self.r[i - 1] + (1.0 + r) * (nose.l[0] + self.l[i]);
        let r = lerp(self.reflection_right, self.new_reflection_right, lambda);
        self.r_[i] = r * self.l[i] + (1.0 + r) * (self.r[i - 1] + nose.l[0]);
        let r = lerp(self.reflection_nose, self.new_reflection_nose, lambda);
        nose.r_[0] = r * nose.l[0] + (1.0 + r) * (self.l[i] + self.r[i - 1]);

        for i in 0..self.length {
            self.r[i] = (self.r_[i] * self.fade).clamp(-1.0, 1.0);
            self.l[i] = (self.l_[i] * self.fade).clamp(-1.0, 1.0);
        }

        self.r[self.length - 1]
    }

    fn reshape(&mut self, amount: F, nose: &Nose) {
        let mut new_last_obstruction = usize::MAX; // indicates whether it is an occlusion
        for i in 0..self.length {
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
            && nose.area[0] < 0.05
        {
            self.add_transient(self.last_obstruction);
        }
        self.last_obstruction = new_last_obstruction;
    }

    fn calculate_reflections(&mut self, nose: &Nose) {
        for i in 0..self.length {
            self.area[i] = self.diameter[i].powi(2); //ignoring PI etc.
        }
        for i in 1..self.length {
            self.reflection[i] = self.new_reflection[i];
            self.new_reflection[i] = if self.area[i] == 0.0 {
                0.999
            } else {
                (self.area[i - 1] - self.area[i]) / (self.area[i - 1] + self.area[i])
            };
        }

        //now at junction with nose
        self.reflection_left = self.new_reflection_left;
        self.reflection_right = self.new_reflection_right;
        self.reflection_nose = self.new_reflection_nose;
        let sum = self.area[self.nose_start] + self.area[self.nose_start + 1] + nose.area[0];
        self.new_reflection_left = 2.0 * self.area[self.nose_start] / sum - 1.0;
        self.new_reflection_right = 2.0 * self.area[self.nose_start + 1] / sum - 1.0;
        self.new_reflection_nose = 2.0 * nose.area[0] / sum - 1.0;
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

    fn process_transients(&mut self, dtime: F) {
        for trans in self.transients.iter_mut() {
            let amplitude = trans.strength * (2 as F).powf(-trans.exponent * trans.time_alive);
            self.r[trans.position] += amplitude * 0.5;
            self.l[trans.position] += amplitude * 0.5;
            trans.time_alive += dtime;
        }
        self.transients.retain(|t| t.time_alive <= t.life_time)
    }

    fn add_turbulence_noise(&mut self, time: f64, turbulence_noise: F) {
        for constriction in &self.other_constrictions {
            if !(2.0..self.length as F).contains(&constriction.index)
                || constriction.diameter <= 0.0
            {
                continue;
            }
            let intensity = constriction.fricative_intensity(time);
            if intensity == 0.0 {
                continue;
            }
            add_turbulence_noise_at_index(
                &mut self.r,
                &mut self.l,
                0.66 * turbulence_noise * intensity,
                constriction.index,
                constriction.diameter,
            );
        }

        // Remove dead constrictions
        self.other_constrictions.retain(|c| {
            if let Some(end_time) = c.end_time {
                time < end_time + 1.0
            } else {
                true
            }
        });
    }

    pub fn calculate_diameter(&mut self) {
        const GRID_OFFSET: F = 1.7;

        let (tongue_index, tongue_diameter) = self.tongue;

        self.target_diameter
            .copy_from_slice(&self.original_diameter);
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
            self.target_diameter[i] = 1.5 - curve;
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
        let out = from_point * 0.5 * (upper_index_bound - lower_index_bound);
        let index = index.clamp(index_center - out, index_center + out);

        let diameter = diameter.clamp(INNER_RADIUS, OUTER_RADIUS);

        (index, diameter)
    }
}

fn add_turbulence_noise_at_index(
    r: &mut [F],
    l: &mut [F],
    turbulence_noise: F,
    index: F,
    diameter: F,
) {
    let i = index.floor() as usize;
    let delta = index - i as F;

    let thinness = (8.0 * (0.7 - diameter)).clamp(0.0, 1.0);
    let openness = (30.0 * (diameter - 0.3)).clamp(0.0, 1.0);
    let noise0 = turbulence_noise * (1.0 - delta) * thinness * openness;
    let noise1 = turbulence_noise * delta * thinness * openness;
    r[i + 1] += noise0 * 0.5;
    l[i + 1] += noise0 * 0.5;
    r[i + 2] += noise1 * 0.5;
    l[i + 2] += noise1 * 0.5;
}

pub struct Nose {
    length: usize,
    diameter: Vec<F>,
    area: Vec<F>,
    r: Vec<F>,
    l: Vec<F>,
    r_: Vec<F>,
    l_: Vec<F>,
    reflection: Vec<F>,

    pub fade: F, // 0.9999
    /// 0.01 - 0.4
    pub velum_target: F,
    lip_reflection: F,
}

impl Nose {
    fn new(length: usize) -> Self {
        Nose {
            length,
            r: vec![0.0; length],
            l: vec![0.0; length],
            r_: vec![0.0; length],
            l_: vec![0.0; length],
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
            area: vec![0.0; length],
            fade: 0.999,
            velum_target: 0.01,
            lip_reflection: -0.85,
        }
    }

    fn run_step(&mut self) -> F {
        self.l_[self.length - 1] = self.r[self.length - 1] * self.lip_reflection;

        for i in 1..self.length {
            let w = self.reflection[i] * (self.r[i - 1] + self.l[i]);
            self.r_[i] = self.r[i - 1] - w;
            self.l_[i - 1] = self.l[i] + w;
        }

        for i in 0..self.length {
            self.r[i] = (self.r_[i] * self.fade).clamp(-1.0, 1.0);
            self.l[i] = (self.l_[i] * self.fade).clamp(-1.0, 1.0);
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
        self.area[0] = self.diameter[0].powi(2);
    }

    fn calculate_reflections(&mut self) {
        for i in 0..self.length {
            self.area[i] = self.diameter[i].powi(2);
        }
        for i in 1..self.length {
            self.reflection[i] =
                (self.area[i - 1] - self.area[i]) / (self.area[i - 1] + self.area[i]);
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

struct Transient {
    position: usize,
    time_alive: F,
    life_time: F,
    strength: F,
    exponent: F,
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
        // TODO: make smooth
        if let Some(end_time) = self.end_time {
            1.0 - (time - end_time) / fricative_attack_time
        } else {
            (time - self.start_time) / fricative_attack_time
        }
        .clamp(0.0, 1.0)
    }
}
