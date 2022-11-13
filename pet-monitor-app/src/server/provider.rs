//! Subscribable interior mutability

use parking_lot::RwLock;
use rocket::tokio::sync::broadcast;
use std::fmt::Debug;
use std::sync::Arc;

/// The `Provider` type implements thread-safe interior mutability. It
/// broadcasts on an MPMC channel each time the inner value is mutated.
#[derive(Debug, Clone)]
pub struct Provider<T> {
    inner: Arc<RwLock<T>>,
    sub: broadcast::Sender<T>,
}

impl<T: Clone> Provider<T> {
    /// Creates a new `Provider`.
    pub fn new(val: T) -> Self {
        Self {
            inner: Arc::new(RwLock::new(val)),
            sub: broadcast::channel::<T>(16).0,
        }
    }

    /// Gets the value stored in the `Provider`.
    pub fn get(&self) -> T {
        self.inner.read().clone()
    }

    /// Replaces the value in the `Provider` with a new value.
    pub fn set(&self, new: T) {
        *self.inner.write() = new.clone();
        self.sub.send(new).unwrap_or(0); // Ignore error if there are no receivers
    }

    #[allow(dead_code)]
    pub fn update<F: FnOnce(&mut T)>(&self, f: F) {
        let mut val = self.get();
        f(&mut val);
        self.set(val);
    }

    /// Returns a `Receiver` that will send every time the inner value is mutated.
    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.sub.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::tokio;

    #[tokio::test]
    async fn test_provider() {
        let val = "foo".to_string();
        let prov = Provider::new(val.clone());
        let mut sub = prov.subscribe();

        assert_eq!(val, prov.get());

        let val = "bar".to_string();
        prov.set(val.clone());

        assert_eq!(val, prov.get());
        assert_eq!(val, sub.recv().await.unwrap());

        let val = "baz".to_string();
        prov.set(val.clone());

        assert_eq!(val, prov.get());
        assert_eq!(val, sub.recv().await.unwrap());
    }
}
