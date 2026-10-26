use std::sync::{Arc, PoisonError, RwLock};

/// An `ArcSwap` stand-in for state that native plugins can replace.
///
/// Native plugins link their own copy of `arc_swap`, whose reader tracking lives in a global
/// static, so a swap done by plugin code cannot see the server's readers and may free a value
/// the server is still using. Here the lock state lives in the value itself, so both copies of
/// the code agree on it. Readers get an owned `Arc` snapshot and never hold the lock.
#[derive(Debug)]
pub struct SharedArcSwap<T>(RwLock<Arc<T>>);

impl<T> SharedArcSwap<T> {
    #[must_use]
    pub const fn new(value: Arc<T>) -> Self {
        Self(RwLock::new(value))
    }

    #[must_use]
    pub fn from_pointee(value: T) -> Self {
        Self::new(Arc::new(value))
    }

    #[must_use]
    pub fn load(&self) -> Arc<T> {
        Arc::clone(&self.0.read().unwrap_or_else(PoisonError::into_inner))
    }

    #[must_use]
    pub fn load_full(&self) -> Arc<T> {
        self.load()
    }

    pub fn store(&self, value: Arc<T>) {
        let old = std::mem::replace(
            &mut *self.0.write().unwrap_or_else(PoisonError::into_inner),
            value,
        );
        drop(old);
    }

    /// Replaces the value with `f(current)` and returns the previous one. `f` runs under the
    /// write lock exactly once, so it must not read this same cell.
    pub fn rcu<R, F>(&self, mut f: F) -> Arc<T>
    where
        F: FnMut(&Arc<T>) -> R,
        R: Into<Arc<T>>,
    {
        let mut current = self.0.write().unwrap_or_else(PoisonError::into_inner);
        let new = f(&current).into();
        std::mem::replace(&mut *current, new)
    }
}
