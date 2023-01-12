//! Utility functions and types

#[derive(Debug)]

/// A wrapper around a spin::Mutex, allowing us to implement GlobalAlloc. 
pub struct WrappedMutex<T> {
    inner: spin::Mutex<T>
}

impl<T> WrappedMutex<T> {
    /// Creates a new WrappedMutex
    pub const fn new(inner: T) -> Self {
        WrappedMutex { inner: spin::Mutex::new(inner) }
    }

    /// Locks the mutex and returns a guard
    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}