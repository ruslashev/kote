// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![no_main]
#![allow(clippy::empty_loop)]

use core::arch::asm;

static STR: &str = "Hello, World!";

fn syscall(num: u64, arg1: u64, arg2: u64, arg3: u64, arg4: u64) -> u64 {
    let ret;

    unsafe {
        asm!(
            "syscall",
            inlateout("rax") num => ret,
            in("rdi") arg1,
            in("rsi") arg2,
            in("rdx") arg3,
            in("r10") arg4,
            out("rcx") _,
            out("r11") _,
            options(nostack),
        );
    }

    ret
}

#[no_mangle]
pub extern "C" fn _start() {
    loop {
        syscall(1, STR.as_ptr() as u64, STR.len() as u64, 0, 0);
        let _bad = syscall(1, 0xffffff8000000000, 800000, 0, 0);
        syscall(0, 0, 0, 0, 0);
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
