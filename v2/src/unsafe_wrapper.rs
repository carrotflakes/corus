use super::PackedEvent;

use std::ops::DerefMut;
use std::sync::Arc;

pub struct UnsafeWrapper<T: Send + Sync + 'static>(Arc<T>);

impl<T: Send + Sync + 'static> UnsafeWrapper<T> {
    pub fn new(inner: T) -> Self {
        Self(Arc::new(inner))
    }

    pub fn make_event(&self, f: impl FnOnce(&mut T, f64) + Send + Sync + 'static) -> PackedEvent {
        let mut inner = self.clone();
        Box::new(move |time| {
            let inner = inner.deref_mut();
            f(inner, time);
        })
    }
}

impl<T: Send + Sync + 'static> std::ops::Deref for UnsafeWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<T: Send + Sync + 'static> std::ops::DerefMut for UnsafeWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute::<*const T, &mut T>(&*self.0) }
    }
}

impl<T: Send + Sync + 'static> Clone for UnsafeWrapper<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

unsafe impl<T: Send + Sync + 'static> Send for UnsafeWrapper<T> {}
unsafe impl<T: Send + Sync + 'static> Sync for UnsafeWrapper<T> {}
