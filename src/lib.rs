/*! A Rust atomic lock type.

This is a simple atomic lock.

There is no way to sleep the current thread if the lock is not available, what you do about that is up to you.
*/

use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct AtomicLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> AtomicLock<T> {
    pub fn new(data: T) -> Self {
        AtomicLock {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    /**
    Locks the lock and accesses the data if available.
    If the lock is unavailable, will return None.
    */
    pub fn lock(&self) -> Option<Guard<T>> {
        match self.lock.compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed) {
            Ok(_) => Some(
                Guard {
                    lock: self,
                    data: unsafe { &mut *self.data.get() },
                }

            ),
            Err(_) => None,
        }
    }

    pub fn unlock(&self) {
        let old = self.lock.swap(false, Ordering::Release);
        assert_eq!(old, true);
    }

    /** Unsafely access the underlying data.

    # Safety
    This function is unsafe because it allows you to access the underlying data without a lock.
    */
    pub unsafe fn data(&self) -> &mut T {
        &mut *self.data.get()
    }

}

pub struct Guard<'a, T> {
    lock: &'a AtomicLock<T>,
    data: &'a mut T,
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
