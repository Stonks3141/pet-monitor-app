//! Async interior mutability with channels

use tokio::sync::{mpsc, oneshot, watch};
use std::fmt::Debug;
use std::future::Future;

/// The `Provider` type uses async channels to implement thread-safe interior
/// mutability. It executes a callback every time the inner value is mutated.
#[derive(Debug, Clone)]
pub struct Provider<T> {
    get: mpsc::Sender<oneshot::Sender<T>>,
    set: mpsc::Sender<(T, oneshot::Sender<()>)>,
    sub: watch::Receiver<T>,
}

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
        let (set, mut set_rx) = mpsc::channel::<(T, oneshot::Sender<()>)>(100);
        let (get, mut get_rx) = mpsc::channel::<oneshot::Sender<T>>(100);
        let (sub_tx, sub) = watch::channel::<T>(val.clone());

        tokio::spawn(async move {
            loop {
                if let Some(response) = get_rx.recv().await {
                    response.send(val.clone()).unwrap();
                }
                if let Some((new, sender)) = set_rx.recv().await {
                    val = new;
                    on_set(val.clone()).await;
                    sub_tx.send(val.clone()).unwrap();
                    // needed to drive this future forward and eagerly execute on_set and broadcast update
                    sender.send(()).unwrap();
                }
            }
        });
        Self {
            get,
            set,
            sub,
        }
    }

    /// Gets the value stored in the `Provider`.
    #[inline]
    pub async fn get(&self) -> T
    where
        T: Debug,
    {
        let (tx, rx) = oneshot::channel();
        self.get.send(tx).await.unwrap();
        rx.await.unwrap()
    }

    /// Replaces the value in the `Provider` with a new value.
    #[inline]
    pub async fn set(&self, new: T)
    where
        T: Debug,
    {
        let (tx, rx) = oneshot::channel();
        self.set.send((new, tx)).await.unwrap();
        rx.await.unwrap();
    }

    /// Returns `Some` if the inner value has changed since the last call to
    /// `poll` or `changed`. Does not block.
    #[inline]
    pub fn poll(&mut self) -> Option<T>
    where
        T: Clone,
    {
        if self.sub.has_changed().unwrap() {
            Some((*self.sub.borrow_and_update()).clone())
        } else {
            None
        }
    }

    /// Awaits for the inner value to change, then returns the new value.
    #[inline]
    pub async fn changed(&mut self) -> T
    where
        T: Clone
    {
        self.sub.changed().await.unwrap();
        (*self.sub.borrow()).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert!(!*mutex.lock().unwrap());

        let val = "bar".to_string();
        prov.set(val.clone()).await;
        assert!(*mutex.lock().unwrap());

        assert_eq!(val, prov.get().await);
    }
}
