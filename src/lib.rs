/*! A Rust atomic lock type.

This is a simple atomic lock.

There is no way to sleep the current thread if the lock is not available, what you do about that is up to you.
*/

use std::cell::UnsafeCell;
use std::fmt::Display;
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug)]
pub struct AtomicLock<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> AtomicLock<T> {
    pub const fn new(data: T) -> Self {
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

#[derive(Debug)]
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


/*Display is tough to do since we can't read the data.  We could display status of the lock... */

impl Display for AtomicLock<()> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "AtomicLock: {}", self.lock.load(Ordering::Relaxed))
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

PartialEq and Eq could be based on the data.
 */

impl <'a, T> PartialEq for Guard<'a, T> where T: PartialEq {
    fn eq(&self, other: &Self) -> bool {
        self.data.eq(&other.data)
    }
}

impl <'a, T> Eq for Guard<'a, T> where T: Eq {}

/* Similarly, partialOrd and ord */

impl <'a, T> PartialOrd for Guard<'a, T> where T: PartialOrd {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl <'a, T> Ord for Guard<'a, T> where T: Ord {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.data.cmp(&other.data)
    }
}

//hash,

impl <'a, T> std::hash::Hash for Guard<'a, T> where T: std::hash::Hash {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state)
    }
}

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

//send is ok while sync is not, we guarantee exclusive access

unsafe impl<'a, T> Send for Guard<'a, T> {}