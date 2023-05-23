[![check](https://github.com/Defelo/key-rwlock/actions/workflows/check.yml/badge.svg)](https://github.com/Defelo/key-rwlock/actions/workflows/check.yml)
[![test](https://github.com/Defelo/key-rwlock/actions/workflows/test.yml/badge.svg)](https://github.com/Defelo/key-rwlock/actions/workflows/test.yml)
[![codecov](https://codecov.io/gh/Defelo/key-rwlock/branch/develop/graph/badge.svg?token=D88MHBLVE1)](https://codecov.io/gh/Defelo/key-rwlock)
![Version](https://img.shields.io/github/v/tag/Defelo/key-rwlock?include_prereleases&label=version)
[![dependency status](https://deps.rs/repo/github/Defelo/key-rwlock/status.svg)](https://deps.rs/repo/github/Defelo/key-rwlock)

# key-rwlock
Simple library for keyed asynchronous reader-writer locks.

## Example
```rust
use key_rwlock::KeyRwLock;

#[tokio::main]
async fn main() {
    let lock = KeyRwLock::new();

    let _foo = lock.write("foo").await;
    let _bar = lock.read("bar").await;

    assert!(lock.try_read("foo").await.is_err());
    assert!(lock.try_write("foo").await.is_err());

    assert!(lock.try_read("bar").await.is_ok());
    assert!(lock.try_write("bar").await.is_err());
}
```
