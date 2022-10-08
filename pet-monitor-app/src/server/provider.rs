//! Async interior mutability

use rocket::tokio::sync::{broadcast, Mutex};
use std::fmt::Debug;
use std::sync::Arc;

/// The `Provider` type implements thread-safe async interior mutability. It
/// broadcasts on an MPMC channel each time the inner value is mutated.
#[derive(Debug, Clone)]
pub struct Provider<T> {
    inner: Arc<Mutex<T>>,
    sub: broadcast::Sender<T>,
}

impl<T> Provider<T> {
    /// Creates a new `Provider`.
    pub fn new(val: T) -> Self
    where
        T: Clone,
    {
        let (sub, _) = broadcast::channel::<T>(100);
        let inner = Arc::new(Mutex::new(val));
        Self { inner, sub }
    }

    /// Gets the value stored in the `Provider`.
    pub async fn get(&self) -> T
    where
        T: Clone,
    {
        let lock = &*self.inner.lock().await;
        lock.clone()
    }

    /// Replaces the value in the `Provider` with a new value.
    pub async fn set(&self, new: T)
    where
        T: Debug + Clone,
    {
        *self.inner.lock().await = new.clone();
        self.sub.send(new).unwrap();
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

        assert_eq!(val, prov.get().await);

        let val = "bar".to_string();
        prov.set(val.clone()).await;

        assert_eq!(val, prov.get().await);
        assert_eq!(val, sub.recv().await.unwrap());
    }
}
