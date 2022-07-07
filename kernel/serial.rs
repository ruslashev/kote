// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::arch::uart;
use crate::spinlock::Mutex;

type SerialImpl = uart::Uart;

pub static SERIAL_LOCK: Mutex<SerialImpl> = Mutex::new(SerialImpl {});

pub trait Serial {
    fn init();

    fn read_byte(&self) -> u8;

    fn write_byte(&self, byte: u8);
}

impl core::fmt::Write for SerialImpl {
    // NOTE: By itself makes no exclusivity guarantees
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for b in s.bytes() {
            self.write_byte(b);
        }

        Ok(())
    }
}

pub fn init() {
    SerialImpl::init();
}
