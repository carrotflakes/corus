pub struct Cache<T, S, F>
where
    T: Clone + PartialEq,
    F: Fn(T) -> S,
{
    cache: Option<(T, S)>,
    f: F,
}

impl<T, S, F> Cache<T, S, F>
where
    T: Clone + PartialEq,
    F: Fn(T) -> S,
{
    pub fn new(f: F) -> Self {
        Self { cache: None, f }
    }

    pub fn update(&mut self, x: T) {
        let y = (self.f)(x.clone());
        self.cache = Some((x, y));
    }

    pub fn get(&mut self, x: T) -> &S {
        if self.cache.as_ref().map(|c| c.0 != x).unwrap_or(true) {
            let y = (self.f)(x.clone());
            self.cache = Some((x, y));
        }
        &self.cache.as_ref().unwrap().1
    }

    pub fn get_ref(&self, x: T) -> Option<&S> {
        if self.cache.as_ref().map(|c| c.0 == x).unwrap_or(false) {
            Some(&self.cache.as_ref().unwrap().1)
        } else {
            None
        }
    }
}
