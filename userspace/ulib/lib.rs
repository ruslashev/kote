// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]

use core::arch::asm;

extern "Rust" {
    fn main();
}

#[no_mangle]
pub extern "C" fn _start(_argc: usize, _argv: usize) {
    unsafe {
        main();
    }
}

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

pub fn write(s: &str) -> u64 {
    syscall(1, s.as_ptr() as u64, s.len() as u64, 0, 0)
}

pub fn sched_yield() {
    syscall(0, 0, 0, 0, 0);
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
