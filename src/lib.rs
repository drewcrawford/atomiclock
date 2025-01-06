
/*! A Rust atomic lock type.

This is a simple atomic lock.

There is no way to sleep the current thread if the lock is not available, what you do about that is up to you.
*/

use std::cell::UnsafeCell;
use std::fmt::{Debug, Display};
use std::sync::atomic::{AtomicBool, Ordering};

/**
An atomic lock type.

*/
pub struct AtomicLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> AtomicLock<T> {
    /**
    Creates a new lock
*/
    pub const fn new(data: T) -> Self {
        AtomicLock {
            lock: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }
    /**
    Locks the lock and accesses the data if available.
    If the lock is unavailable, will return None.

    There is intentionally nothing 'to do' about this, as far as this crate is concerned.
    Other crates may wrap this type and implement some other behavior.  For example:
    * You could spin, creating a spinlock
    * You could sleep, creating an OS lock somehow
    * You could yield, creating a cooperative async lock

    It's up to you!
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

    /**
    Unlocks the current lock.
*/
    pub fn unlock(&self) {
        let old = self.lock.swap(false, Ordering::Release);
        assert_eq!(old, true);
    }

    /** Unsafely access the underlying data.

    # Safety
    You must ensure that no other readers or writers are accessing the lock.
    */
    pub unsafe fn data(&self) -> &mut T {
        &mut *self.data.get()
    }

    /**
    Conumes the lock, returning the inner data.
    */
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

}


impl<T: Debug> Debug for AtomicLock<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.lock();
        match guard {
            None => {
                f.debug_struct("AtomicLock")
                    .field("locked", &true)
                    .field("data", &"<Locked>")
                    .finish()
            }
            Some(data) => {
                f.debug_struct("AtomicLock")
                    .field("locked", &false)
                    .field("data", &data)
                    .finish()
            }
        }
    }
}

/**
A guard for [AtomicLock].

Unlocks when dropped.
*/

#[derive(Debug)]
#[must_use]
pub struct Guard<'a, T> {
    lock: &'a AtomicLock<T>,
    data: &'a mut T,
}

impl<'a, T> Drop for Guard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

//boilerplate
/*
I think we don't want to derive Clone, reading the data would involve acquiring the lock...
same for PartialEq, PartialOrd, etc.
Same for hash...
default maybe?
 */

impl <T> Default for AtomicLock<T> where T: Default {
    fn default() -> Self {
        AtomicLock::new(T::default())
    }
}


//from is probably fine

impl <T> From<T> for AtomicLock<T> {
    fn from(data: T) -> Self {
        AtomicLock::new(data)
    }
}


//asref/mut requires owning the data, so nogo
//same for deref / derefmut

//send and sync are ok

unsafe impl<T> Send for AtomicLock<T> {}
unsafe impl<T> Sync for AtomicLock<T> {}

/*now let's examine the guard boilerplate.

Guard cannot be cloned because doing so would involve locking a second time.  Similarly, not copy.

 */


//default makes no sense and should not be done

//display,

impl <'a, T> Display for Guard<'a, T> where T: Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt(f)
    }
}

//from and into make no sense.

//asref/mut should be fine since we have the lock...

impl <'a, T> AsRef<T> for Guard<'a, T> {
    fn as_ref(&self) -> &T {
        self.data
    }
}

impl <'a, T> AsMut<T> for Guard<'a, T> {
    fn as_mut(&mut self) -> &mut T {
        self.data
    }
}

//deref/derefmut should be fine since we have the lock...

impl <'a, T> std::ops::Deref for Guard<'a, T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.data
    }
}

impl <'a, T> std::ops::DerefMut for Guard<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}

/*
Send is ok.

MutexGuard does not implement Send, due to OS constraints on unlocking from the same thread
as locked.

We don't have those issues, so.
 */
unsafe impl<'a, T> Send for Guard<'a, T> {}