#![no_std]
#![feature(asm)]
#![allow(dead_code)]

#[cfg(target_arch = "x86_64")]
#[path = "arch/x64/mod.rs"]
mod arch;

mod consts;
mod panic;

#[no_mangle]
pub fn kmain() -> !
{
	panic::panic_early("oops!");

	loop {}
}
