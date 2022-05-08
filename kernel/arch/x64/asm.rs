// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::arch::asm;

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
