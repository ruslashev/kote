// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::mm::types::VirtAddr;

#[macro_use]
mod asm;
pub mod backtrace;
pub mod interrupts;
pub mod mmu;
pub mod uart;

pub const KERNEL_BASE: u64 = 0xffffff8000000000;

pub const USER_STACK_START: VirtAddr = VirtAddr(0x0000001000000000);
pub const USER_STACK_SIZE: usize = 4 * mmu::PAGE_SIZE;

const GDT_KERN_CODE: u8 = 8;
const GDT_KERN_DATA: u8 = 16;
const GDT_USER_CODE: u8 = 24;
const GDT_USER_DATA: u8 = 32;

pub type RegisterFrame = interrupts::exceptions::ExceptionFrame;
pub type RootPageDir = mmu::PageMapLevel4;
pub type LeafDirEntry = mmu::PageTableEntry;
pub type LeafDirEntryLarge = mmu::PageDirectoryEntry;
