// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::arch::asm;

pub mod exceptions;
mod handlers;
mod idt;
mod pic;
mod rtc;

pub fn init() {
    pic::remap();
    idt::init();
    rtc::init();

    pic::enable_line(2);
    pic::enable_line(8);
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

pub fn with_disabled<F: FnMut()>(mut f: F) {
    let flags = save_flags_cli();
    f();
    restore_flags(flags);
}

#[inline(always)]
fn save_flags_cli() -> u64 {
    let flags;

    unsafe {
        asm!("pushf",
             "cli",
             "pop {}",
             out(reg) flags);
    }

    flags
}

#[inline(always)]
fn restore_flags(flags: u64) {
    unsafe {
        asm!("push {}",
             "popf",
             in(reg) flags);
    }
}
