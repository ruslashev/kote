// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::arch::asm;

use crate::mm::types::VirtAddr;

pub mod io {
    use super::asm;

    #[inline(always)]
    pub fn outb(port: u16, val: u8) {
        unsafe {
            asm!("out dx, al",
                in("dx") port,
                in("al") val,
                options(nostack, preserves_flags));
        }
    }

    #[inline(always)]
    pub fn inb(port: u16) -> u8 {
        let ret: u8;

        unsafe {
            asm!("in al, dx",
                in("dx") port,
                out("al") ret,
                options(nostack, preserves_flags));
        }

        ret
    }

    #[inline(always)]
    pub fn wait() {
        outb(0x80, 0); // unused port
    }
}

pub fn invalidate_dcache(addr: VirtAddr) {
    unsafe {
        asm!("invlpg [{}]",
            in(reg) addr.0,
            options(nostack, preserves_flags));
    }
}

pub fn nmi_enable() {
    io::outb(0x70, io::inb(0x70) & !(1 << 7));
    io::inb(0x71);
}

pub fn nmi_disable() {
    io::outb(0x70, io::inb(0x70) | (1 << 7));
    io::inb(0x71);
}

macro_rules! read_reg {
    ($reg:ident) => {
        unsafe {
            use core::arch::asm;

            let val: u64;

            asm!(concat!("mov {}, ", stringify!($reg)),
                out(reg) val,
                options(nomem, nostack));

            val
        }
    }
}

macro_rules! read_fp {
    () => {
        read_reg!(rbp)
    };
}

macro_rules! write_reg {
    ($reg:ident, $val:expr) => {
        unsafe {
            use core::arch::asm;

            let val: u64 = $val;

            asm!(concat!("mov ", stringify!($reg), ", {}"),
                in(reg) val,
                options(nomem, nostack));
        }
    }
}

pub fn idle() {
    unsafe {
        asm!("hlt");
    }
}
