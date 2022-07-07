// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

struct Spinlock {
    locked: AtomicBool,
}

impl Spinlock {
    const fn new() -> Self {
        Spinlock {
            locked: AtomicBool::new(false),
        }
    }

    fn lock(&self) {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err()
        {
            core::hint::spin_loop();
        }
    }

    fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

pub struct SpinlockMutex<T: ?Sized> {
    marker: PhantomData<T>,
    lock: Spinlock,
    data: UnsafeCell<T>,
}

pub type Mutex<T> = SpinlockMutex<T>;

impl<T> SpinlockMutex<T> {
    pub const fn new(data: T) -> Self {
        SpinlockMutex {
            marker: PhantomData,
            data: UnsafeCell::new(data),
            lock: Spinlock::new(),
        }
    }

    pub fn guard(&self) -> SpinlockGuard<T> {
        self.lock.lock();

        SpinlockGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }

    /// Used as last resort (e.g. before/during panicking) and leaves spinlock in unusable state
    pub fn force_unlock(&self) -> SpinlockGuard<T> {
        SpinlockGuard {
            lock: &self.lock,
            data: unsafe { &mut *self.data.get() },
        }
    }
}

unsafe impl<T> Send for SpinlockMutex<T> {}

unsafe impl<T> Sync for SpinlockMutex<T> {}

pub struct SpinlockGuard<'a, T> {
    lock: &'a Spinlock,
    pub data: &'a mut T,
}

impl<'a, T> Drop for SpinlockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

impl<T> Deref for SpinlockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.data
    }
}

impl<T> DerefMut for SpinlockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        self.data
    }
}
