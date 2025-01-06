# atomiclock

A Rust atomic lock type.

![logo](art/logo.png)

This is a simple atomic lock.

There is no way to sleep the current thread if the lock is not available, what you do about that is up to you.