//! Async interior mutability with channels

use rocket::tokio;
use rocket::tokio::sync::{mpsc, oneshot};
use std::fmt::Debug;
use std::future::Future;

/// The `Provider` type uses async channels to implement thread-safe interior
/// mutability. It executes a callback every time the inner value is mutated.
#[derive(Debug, Clone)]
pub struct Provider<T>(mpsc::Sender<(Option<T>, oneshot::Sender<T>)>);

impl<T> Provider<T> {
    /// Creates a new `Provider`.
    ///
    /// The `on_set` callback will be run with the new value whenever
    /// `Provider::set` is called.
    pub fn new<F, Fut>(mut val: T, mut on_set: F) -> Self
    where
        T: Clone + Send + Sync + Debug + 'static,
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send,
    {
        let (tx, mut rx) = mpsc::channel::<(Option<T>, oneshot::Sender<T>)>(100);
        tokio::spawn(async move {
            while let Some((new, response)) = rx.recv().await {
                if let Some(new) = new {
                    let prev = val.clone();
                    val = new;
                    on_set(val.clone()).await;
                    response.send(prev).unwrap();
                } else {
                    response.send(val.clone()).unwrap();
                }
            }
        });
        Self(tx)
    }

    /// Gets the value stored in the `Provider`.
    #[inline]
    pub async fn get(&self) -> T
    where
        T: Debug,
    {
        let (tx, rx) = oneshot::channel();
        self.0.send((None, tx)).await.unwrap();
        rx.await.unwrap()
    }

    /// Replaces the value in the `Provider` with a new value.
    #[inline]
    pub async fn set(&self, new: T)
    where
        T: Debug,
    {
        let (tx, rx) = oneshot::channel();
        self.0.send((Some(new), tx)).await.unwrap();
        rx.await.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rocket::tokio;
    use std::sync::{Arc, Mutex};

    #[tokio::test]
    async fn test_provider() {
        let val = "foo".to_string();
        let mutex = Arc::new(Mutex::new(false));
        let mutex_clone = mutex.clone();

        let prov = Provider::new(val.clone(), move |_| {
            let mutex_clone = mutex_clone.clone();
            async move {
                *mutex_clone.lock().unwrap() = true;
            }
        });

        assert_eq!(val, prov.get().await);
        assert_eq!(false, *mutex.lock().unwrap());

        let val = "bar".to_string();
        prov.set(val.clone()).await;

        assert_eq!(val, prov.get().await);
        assert_eq!(true, *mutex.lock().unwrap());
    }
}
