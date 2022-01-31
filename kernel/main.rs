// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![allow(dead_code)]

#[macro_use]
mod printk;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x64/mod.rs"]
mod arch;

mod consts;
mod panic;
mod serial;
mod spinlock;

#[no_mangle]
pub fn kmain() -> ! {
    serial::init();

    printk!("Hello, World! {} + {} = {}\n", 1, 2, 1 + 2);

    panic!("oops!");

    loop {}
}
