#![no_std]
#![feature(asm)]
#![allow(dead_code)]

#[cfg(target_arch = "x86_64")]
#[path = "arch/x64/mod.rs"]
mod arch;

mod consts;
mod panic;
mod printk;
mod serial;
mod spinlock;

#[no_mangle]
pub fn kmain() -> !
{
	serial::init();

	printk!("Hello, World! {} + {} = {}\n", 1, 2, 1 + 2);

	loop {}
}
