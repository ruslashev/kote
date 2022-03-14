// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![allow(dead_code)]
#![allow(clippy::empty_loop)]
#![allow(clippy::identity_op)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(fn_traits)]
#![feature(once_cell)]

#[macro_use]
mod printk;

#[macro_use]
mod utils;

#[cfg(target_arch = "x86_64")]
#[path = "arch/x64/mod.rs"]
mod arch;

mod bootloader;
mod console;
mod consts;
mod dwarf;
mod elf;
mod mm;
mod panic;
mod serial;
mod spinlock;

#[no_mangle]
pub extern "C" fn kmain() {
    serial::init();
    let info = bootloader::get_info();
    console::init(&info);
    arch::interrupts::init();
    mm::init();

    arch::interrupts::enable();

    println!("Booting ree...");

    println!("Available memory:");
    print!("{}", info.memory_map.as_ref().unwrap());

    println!("Kernel sections:");
    print!("{}", info.section_headers.as_ref().unwrap());

    use core::arch::asm;
    unsafe {
        asm!("mov rax, [0xffffffff90000000]");
    }
}
