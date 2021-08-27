#![no_std]

mod panic;

const KERNEL_BASE: u64 = 0xffffffff80000000;

#[no_mangle]
pub fn kmain() -> !
{
	let vga = (KERNEL_BASE + 0xb8000) as *mut u16;

	unsafe {
		*vga.offset(0) = 0x0728;
		*vga.offset(1) = 0x073a;
	}

	loop {}
}
