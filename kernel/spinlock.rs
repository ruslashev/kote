// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// A very simple, and far from being the best, spinlock. A good implementation should hold
// owndership of the data that is supposed to have a mutually exlusive access.

use core::sync::atomic::{AtomicBool, Ordering};

pub struct Spinlock {
    locked: AtomicBool,
}

impl Spinlock {
    pub const fn new() -> Self {
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

    pub fn guard(&self) -> SpinlockGuard {
        self.lock();

        SpinlockGuard { lock: self }
    }
}

pub struct SpinlockGuard<'a> {
    lock: &'a Spinlock,
}

impl<'a> Drop for SpinlockGuard<'a> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}
