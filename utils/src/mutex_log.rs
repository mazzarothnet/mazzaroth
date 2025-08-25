use std::sync::{LockResult, Mutex, MutexGuard, PoisonError};
use log::{info, warn};

pub struct StdMutexLog<T> {
    inner: Mutex<T>,
    name: &'static str,
}

impl<T> StdMutexLog<T> {
    pub fn new(inner: Mutex<T>, name: &'static str) -> Self {
        Self { inner, name }
    }

    pub fn lock(&self) -> LockResult<LoggingMutexGuard<'_, T>> {
        info!("[{}] Locking mutex", self.name);
        match self.inner.lock() {
            Ok(guard) => Ok(LoggingMutexGuard {
                guard,
                name: self.name,
            }),
            Err(e) => {
                warn!("[{}] Poisoned mutex", self.name);
                Err(PoisonError::new(LoggingMutexGuard {
                    guard: e.into_inner(),
                    name: self.name,
                }))
            }
        }
    }
}

pub struct LoggingMutexGuard<'a, T> {
    guard: MutexGuard<'a, T>,
    name: &'static str,
}

impl<'a, T> std::ops::Deref for LoggingMutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.guard
    }
}

impl<'a, T> std::ops::DerefMut for LoggingMutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.guard
    }
}

impl<'a, T> Drop for LoggingMutexGuard<'a, T> {
    fn drop(&mut self) {
        info!("[{}] Unlocked mutex", self.name);
    }
}
