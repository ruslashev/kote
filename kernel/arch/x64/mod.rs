// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[macro_use]
mod asm;
pub mod backtrace;
pub mod interrupts;
pub mod mmu;
pub mod uart;

pub const KERNEL_BASE: u64 = 0xffffffff80000000;

pub const GDT_KERN_CODE: u8 = 8;
pub const GDT_KERN_DATA: u8 = 16;
pub const GDT_USER_CODE: u8 = 24;
pub const GDT_USER_DATA: u8 = 32;
