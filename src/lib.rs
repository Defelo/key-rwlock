#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(clippy::dbg_macro, clippy::use_debug)]
#![warn(missing_docs, missing_debug_implementations, clippy::todo)]

use std::{
    collections::HashMap,
    hash::Hash,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use tokio::sync::{Mutex, OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock, TryLockError};

/// An async reader-writer lock, that locks based on a key, while allowing other
/// keys to lock independently. Based on a [HashMap] of [RwLock]s.
#[derive(Debug)]
pub struct KeyRwLock<K> {
    /// The inner map of locks for specific keys.
    locks: Mutex<HashMap<K, Arc<RwLock<()>>>>,
    /// Number of lock accesses.
    accesses: AtomicUsize,
}

impl<K> Default for KeyRwLock<K> {
    fn default() -> Self {
        Self {
            locks: Mutex::default(),
            accesses: AtomicUsize::default(),
        }
    }
}

impl<K> KeyRwLock<K>
where
    K: Eq + Hash + Send + Clone,
{
    /// Create new instance of a [KeyRwLock]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Lock this key with shared read access, returning a guard. Cleans up
    /// locks every 1000 accesses.
    pub async fn read(&self, key: K) -> OwnedRwLockReadGuard<()> {
        let mut locks = self.locks.lock().await;

        if self.accesses.fetch_add(1, Ordering::Relaxed) % 1000 == 0 {
            Self::clean_up(&mut locks);
        }

        let lock = locks.entry(key).or_default().clone();
        drop(locks);

        lock.read_owned().await
    }

    /// Lock this key with exclusive write access, returning a guard. Cleans up
    /// locks every 1000 accesses.
    pub async fn write(&self, key: K) -> OwnedRwLockWriteGuard<()> {
        let mut locks = self.locks.lock().await;

        if self.accesses.fetch_add(1, Ordering::Relaxed) % 1000 == 0 {
            Self::clean_up(&mut locks);
        }

        let lock = locks.entry(key).or_default().clone();
        drop(locks);

        lock.write_owned().await
    }

    /// Try lock this key with shared read access, returning immediately. Cleans
    /// up locks every 1000 accesses.
    pub async fn try_read(&self, key: K) -> Result<OwnedRwLockReadGuard<()>, TryLockError> {
        let mut locks = self.locks.lock().await;

        if self.accesses.fetch_add(1, Ordering::Relaxed) % 1000 == 0 {
            Self::clean_up(&mut locks);
        }

        let lock = locks.entry(key).or_default().clone();
        drop(locks);

        lock.try_read_owned()
    }

    /// Try lock this key with exclusive write access, returning immediately.
    /// Cleans up locks every 1000 accesses.
    pub async fn try_write(&self, key: K) -> Result<OwnedRwLockWriteGuard<()>, TryLockError> {
        let mut locks = self.locks.lock().await;

        if self.accesses.fetch_add(1, Ordering::Relaxed) % 1000 == 0 {
            Self::clean_up(&mut locks);
        }

        let lock = locks.entry(key).or_default().clone();
        drop(locks);

        lock.try_write_owned()
    }

    /// Clean up by removing locks that are not locked.
    pub async fn clean(&self) {
        let mut locks = self.locks.lock().await;
        Self::clean_up(&mut locks);
    }

    /// Remove locks that are not locked currently.
    fn clean_up(locks: &mut HashMap<K, Arc<RwLock<()>>>) {
        let mut to_remove = Vec::new();
        for (key, lock) in locks.iter() {
            if lock.try_write().is_ok() {
                to_remove.push(key.clone());
            }
        }
        for key in to_remove {
            locks.remove(&key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_funcionality() {
        let lock = KeyRwLock::new();

        let _foo = lock.write("foo").await;
        let _bar = lock.read("bar").await;

        assert!(lock.try_read("foo").await.is_err());
        assert!(lock.try_write("foo").await.is_err());

        assert!(lock.try_read("bar").await.is_ok());
        assert!(lock.try_write("bar").await.is_err());
    }

    #[tokio::test]
    async fn test_clean_up() {
        let lock = KeyRwLock::new();
        let _foo_write = lock.write("foo_write").await;
        let _bar_write = lock.write("bar_write").await;
        let _foo_read = lock.read("foo_read").await;
        let _bar_read = lock.read("bar_read").await;
        assert_eq!(lock.locks.lock().await.len(), 4);
        drop(_foo_read);
        drop(_bar_write);
        lock.clean().await;
        assert_eq!(lock.locks.lock().await.len(), 2);
    }
}
