use super::{Node, ProcContext};

pub struct Impulse<T>
where
    T: Clone + 'static + Default,
{
    pub value: T,
    pub fired: bool,
}

impl<T> Impulse<T>
where
    T: Clone + 'static + Default,
{
    pub fn new(value: T) -> Self {
        Impulse {
            value,
            fired: false,
        }
    }
}

impl<T> Node<T> for Impulse<T>
where
    T: Clone + 'static + Default,
{
    #[inline]
    fn proc(&mut self, _ctx: &ProcContext) -> T {
        if self.fired {
            Default::default()
        } else {
            self.fired = true;
            self.value.clone()
        }
    }

    fn lock(&mut self) {}

    fn unlock(&mut self) {}
}

impl<T> AsMut<Self> for Impulse<T>
where
    T: Clone + 'static + Default,
{
    fn as_mut(&mut self) -> &mut Impulse<T> {
        self
    }
}
