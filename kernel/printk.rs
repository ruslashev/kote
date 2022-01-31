// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::serial;
use crate::spinlock::Spinlock;

pub static PRINT_LOCK: Spinlock = Spinlock::new();

pub struct Serial<'a>(&'a Spinlock);

impl Serial<'_> {
    pub fn get() -> Self {
        Serial(&PRINT_LOCK)
    }
}

impl core::fmt::Write for Serial<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.0.lock();

        for b in s.bytes() {
            serial::write_byte(b);
        }

        self.0.unlock();

        Ok(())
    }
}

#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => ({
        use core::fmt::Write;

        let mut serial = $crate::printk::Serial::get();

        write!(&mut serial, "{}\n", format_args!($($arg)*)).unwrap()
    });
}
