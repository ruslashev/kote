// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// A very simple, and far from being the best, spinlock. A good implementation should hold
// owndership of the data that is supposed to have a mutually exlusive access.

use core::cell::UnsafeCell;
use core::marker::PhantomData;
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
