// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![allow(dead_code)]
#![allow(clippy::empty_loop)]
#![feature(once_cell)]

#[macro_use]
mod printk;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x64/mod.rs"]
mod arch;

mod console;
mod consts;
mod multiboot;
mod panic;
mod serial;
mod spinlock;
mod utils;

#[no_mangle]
pub fn kmain() -> ! {
    serial::init();
    let info = multiboot::parse();
    console::init(&info);

    println!("Hello, World! {} + {} = {}", 1, 2, 1 + 2);

    println!("lole");

    loop {}
}
