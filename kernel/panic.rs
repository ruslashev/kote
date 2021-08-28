use core::panic::PanicInfo;
use core::ptr::write_volatile;

use crate::consts::KERNEL_BASE;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    printk!("{}\n", info);

    loop {}
}

/// For early boot stage when we don't have serial or graphics.
pub fn panic_early(message: &str) {
    let vga = (KERNEL_BASE + 0xb8000) as *mut u16;
    let color: u16 = 0x07; // black BG, light gray FG

    for (i, b) in message.bytes().enumerate() {
        unsafe {
            write_volatile(vga.add(i), (color << 8) | (b as u16));
        }
    }

    loop {}
}
