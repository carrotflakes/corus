use super::PackedEvent;

use std::cell::UnsafeCell;
use std::sync::Arc;

pub struct UnsafeWrapper<T>(Arc<UnsafeCell<T>>);

impl<T: 'static> UnsafeWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self(Arc::new(UnsafeCell::new(inner)))
    }

    pub fn make_event(&self, f: impl FnOnce(&mut T, f64) + 'static) -> PackedEvent {
        let inner = self.0.clone();
        Box::new(move |time| {
            let inner = unsafe { &mut *inner.get() };
            f(inner, time);
        })
    }
}

impl<T> std::ops::Deref for UnsafeWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.get() }
    }
}

impl<T> std::ops::DerefMut for UnsafeWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.get() }
    }
}

impl<T> Clone for UnsafeWrapper<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
