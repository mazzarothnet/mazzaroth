use log::info;
use std::sync::{LockResult, MutexGuard, PoisonError};

pub struct Mutex<T> {
    inner: std::sync::Mutex<T>,
    name: &'static str,
}

impl<T> Mutex<T> {
    pub fn new(inner: T, name: &'static str) -> Self {
        let mutex = std::sync::Mutex::new(inner);
        Self { inner: mutex, name }
    }

    pub fn lock(&self) -> LockResult<LoggingMutexGuard<'_, T>> {
        #[cfg(feature = "log_mutex")]
        info!("[{}] Locking mutex", self.name);
        match self.inner.lock() {
            Ok(guard) => Ok(LoggingMutexGuard {
                guard,
                name: self.name,
            }),
            Err(e) => {
                info!("[{}] Poisoned mutex", self.name);
                Err(PoisonError::new(LoggingMutexGuard {
                    guard: e.into_inner(),
                    name: self.name,
                }))
            }
        }
    }
}

#[allow(dead_code)]
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
        #[cfg(feature = "log_mutex")]
        info!("[{}] Unlocked mutex", self.name);
    }
}
