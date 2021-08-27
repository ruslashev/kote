#![no_std]
#![feature(asm)]
#![allow(dead_code)]

#[cfg(target_arch = "x86_64")]
#[path = "arch/x64/mod.rs"]
mod arch;

mod consts;
mod panic;
mod serial;

#[no_mangle]
pub fn kmain() -> !
{
	let mut s = crate::serial::Serial::new();

	s.write_string("Hello, World!\n");

	use core::fmt::Write;
	write!(s, "{} + {} = {}\n", 1, 2, 1 + 2).unwrap();

	loop {}
}
