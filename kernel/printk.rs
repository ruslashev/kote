// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::serial;
use crate::spinlock::Spinlock;

pub static PRINT_LOCK: Spinlock = Spinlock::new();

pub struct Serial;

impl core::fmt::Write for Serial {
    // NOTE: By itself makes no exclusivity guarantees
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            serial::write_byte(b);
        }

        Ok(())
    }
}

#[macro_export]
macro_rules! printk {
    ($($arg:tt)*) => ({
        use core::fmt::Write;
        use $crate::printk::{PRINT_LOCK, Serial};

        let mut serial = Serial;

        PRINT_LOCK.lock();

        write!(&mut serial, "{}\n", format_args!($($arg)*)).unwrap();

        PRINT_LOCK.unlock();
    });
}
