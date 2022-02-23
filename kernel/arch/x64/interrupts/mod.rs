// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::arch::asm;

mod exceptions;
mod handlers;
mod idt;
mod irq;
mod pic;

pub fn init() {
    pic::remap();

    idt::build();
    idt::load();
}

#[inline(always)]
pub fn enable() {
    unsafe {
        asm!("sti", options(nostack));
    }
}

#[inline(always)]
pub fn disable() {
    unsafe {
        asm!("cli", options(nostack));
    }
}
