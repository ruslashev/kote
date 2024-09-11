// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![no_std]
#![feature(format_args_nl)]

#[cfg(target_arch = "x86_64")]
use core::arch::asm;

#[macro_use]
pub mod print;

extern "Rust" {
    fn main();
}

#[no_mangle]
pub extern "C" fn _start(_argc: usize, _argv: usize) {
    unsafe {
        main();
    }
}

#[cfg(target_arch = "aarch64")]
fn syscall(_num: u64, _arg1: u64, _arg2: u64, _arg3: u64, _arg4: u64) -> u64 {
    0
}

#[cfg(target_arch = "x86_64")]
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

pub fn sched_yield() {
    syscall(0, 0, 0, 0, 0);
}

pub fn write(s: &str) -> u64 {
    syscall(1, s.as_ptr() as u64, s.len() as u64, 0, 0)
}

pub fn getch(echo: bool) -> u64 {
    let ch = syscall(2, 0, 0, 0, 0);

    if echo {
        let c = ch as u8 as char;
        let t = &mut [0];
        let s = c.encode_utf8(t);

        write(s);
    }

    ch
}

pub fn readline<'s>() -> &'s str {
    static mut BUF: [u8; 128] = [0; 128];
    let mut idx = 0;

    unsafe {
        while idx < BUF.len() {
            let ch = getch(true) as u8;

            // Enter
            if ch == 13 {
                break;
            }

            BUF[idx] = ch;
            idx += 1;
        }

        core::str::from_utf8(&BUF).unwrap_or("invalid input")
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);

    loop {}
}
