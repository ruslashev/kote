#![no_std]

mod consts;
mod panic;

#[no_mangle]
pub fn kmain() -> !
{
	panic::panic_early("oops!");

	loop {}
}
