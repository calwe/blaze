//! Utility functions and types

use crate::{print, println};

#[derive(Debug)]

/// A wrapper around a spin::Mutex, allowing us to implement GlobalAlloc.
pub struct WrappedMutex<T> {
    inner: spin::Mutex<T>,
}

impl<T> WrappedMutex<T> {
    /// Creates a new WrappedMutex
    pub const fn new(inner: T) -> Self {
        WrappedMutex {
            inner: spin::Mutex::new(inner),
        }
    }

    /// Locks the mutex and returns a guard
    pub fn lock(&self) -> spin::MutexGuard<T> {
        self.inner.lock()
    }
}

/// Trace a 16Bx16B block of memory
pub fn trace_mem_16(location: u64) {
    for i in 0..16 {
        print!("{:016x}: ", location + (i * 16));
        for j in 0..16 {
            let addr = (location + (i * 16 + j)) as u64 as *const u8;
            let val = unsafe { *addr };
            print!("{:02x} ", val);
        }
        println!();
    }
}
