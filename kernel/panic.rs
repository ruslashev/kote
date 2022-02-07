// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::panic::PanicInfo;
use core::ptr::write_volatile;

use crate::consts::KERNEL_BASE;

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

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
