#[derive(Debug)]
pub struct WrappedMutex<T> {
    inner: spin::Mutex<T>
}

impl<T> WrappedMutex<T> {
    pub const fn new(inner: T) -> Self {
        WrappedMutex { inner: spin::Mutex::new(inner) }
    }

    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}