// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]
#![allow(clippy::empty_loop)]

use core::arch::asm;

#[no_mangle]
pub extern "C" fn _start() {
    loop {
        unsafe {
            #[cfg(target_arch = "aarch64")]
            asm!("brk 0");

            #[cfg(target_arch = "x86_64")]
            asm!("int3");
        }
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
