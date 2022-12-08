use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct ArcRwLock<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> ArcRwLock<T> {
    pub fn new(val: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(val)),
        }
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        self.inner.write()
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.inner.read()
    }

    pub fn read_only(&self) -> ReadOnly<T> {
        ReadOnly {
            inner: self.inner.clone(),
        }
    }
}

pub struct ReadOnly<T> {
    inner: Arc<RwLock<T>>,
}

impl<T> ReadOnly<T> {
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.inner.read()
    }
}