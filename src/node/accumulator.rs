use super::{Node, ProcContext};

#[derive(Debug, Clone)]
pub struct Event {
    time: f64,
    value: f32,
}

pub struct Accumulator<A, DA>
where
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
{
    events: Vec<Event>,
    node: DA,
    value: f32,
    upper: f32,
    _a: std::marker::PhantomData<A>,
}

impl<A, DA> Accumulator<A, DA>
where
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
{
    pub fn new(node: DA, upper: f32) -> Self {
        Accumulator {
            events: vec![],
            node,
            value: 0.0,
            upper,
            _a: Default::default(),
        }
    }

    pub fn set_value_at_time(&mut self, time: f64, value: f32) {
        let event = Event { time, value };
        for (i, e) in self.events.iter().enumerate() {
            if time < e.time {
                self.events.insert(i, event);
                return;
            }
        }
        self.events.push(event);
    }
}

impl<A, DA> Node<f32> for Accumulator<A, DA>
where
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
{
    fn proc(&mut self, ctx: &ProcContext) -> f32 {
        let d = self.node.as_mut().proc(ctx) / ctx.sample_rate as f32;
        self.value = self.value + d;

        while !self.events.is_empty() {
            if ctx.time < self.events[0].time {
                break;
            }
            self.value = self.events[0].value;
            self.events.remove(0);
        }

        self.value = self.value.rem_euclid(self.upper);
        self.value
    }

    fn lock(&mut self) {
        self.node.as_mut().lock();
    }

    fn unlock(&mut self) {
        self.node.as_mut().unlock();
    }
}

impl<A, DA> AsMut<Self> for Accumulator<A, DA>
where
    A: Node<f32> + ?Sized,
    DA: AsMut<A>,
{
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[test]
fn test() {
    let mut accumulator = Accumulator::new(super::constant::Constant::new(1.0), 4.0);
    let mut pc = ProcContext::new(4);

    accumulator.set_value_at_time(0.0, 1.0);
    accumulator.set_value_at_time(2.0, 0.5);
    accumulator.set_value_at_time(3.0, -1.0);

    for _ in 0..20 {
        dbg!(pc.time);
        dbg!(accumulator.proc(&pc));
        pc.time += 1.0 / pc.sample_rate as f64;
    }
}
