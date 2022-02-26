// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![allow(dead_code)]
#![allow(clippy::empty_loop)]
#![allow(clippy::identity_op)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(fn_traits)]
#![feature(once_cell)]

#[macro_use]
mod printk;

#[macro_use]
mod utils;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x64/mod.rs"]
mod arch;

mod console;
mod consts;
mod dwarf;
mod elf;
mod multiboot;
mod panic;
mod serial;
mod spinlock;

#[no_mangle]
pub extern "C" fn kmain() {
    serial::init();
    let info = multiboot::parse();
    console::init(&info);
    arch::interrupts::init();
    dwarf::init(&info);

    arch::interrupts::enable();

    println!("Hello, World! {} + {} = {}", 1, 2, 1 + 2);

    println!("lole");

    call1();

    println!("continue");
}

fn call1() {
    call2();
}

fn call2() {
    let _big = [16; 123];

    call3();
}

fn call3() {
    print_backtrace!();

    panic!("uh oh");

    use core::arch::asm;
    unsafe {
        asm!("mov eax, 1", "mov ecx, 0", "div ecx");
    }
}
