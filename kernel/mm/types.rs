// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;

use crate::arch::{self, LeafDirEntry, LeafDirEntryLarge};
use crate::bootloader::BootloaderInfo;
use crate::mm::pg_alloc::{self, PageInfo};
use crate::types::PowerOfTwoOps;

#[derive(Copy, Clone, Debug)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct VirtAddr(pub usize);

pub trait Address: From<usize> {
    fn from_u64(raw: u64) -> Self {
        Self::from(raw.try_into().expect("Address: u64 overflows usize"))
    }

    fn from_u32(raw: u32) -> Self {
        Self::from(raw.try_into().expect("Address: u32 overflows usize"))
    }
}

impl From<usize> for PhysAddr {
    fn from(scalar: usize) -> Self {
        Self(scalar)
    }
}

impl From<usize> for VirtAddr {
    fn from(scalar: usize) -> Self {
        Self(scalar)
    }
}

impl core::ops::Add<usize> for PhysAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl core::ops::Add<usize> for VirtAddr {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

impl Address for PhysAddr {}

impl Address for VirtAddr {}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x?}", self)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x?}", self)
    }
}

impl fmt::LowerHex for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl fmt::LowerHex for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::LowerHex::fmt(&self.0, f)
    }
}

impl From<PhysAddr> for VirtAddr {
    fn from(paddr: PhysAddr) -> Self {
        VirtAddr(paddr.0 + arch::KERNEL_BASE as usize)
    }
}

impl PhysAddr {
    pub fn into_vaddr(self) -> VirtAddr {
        self.into()
    }

    pub fn dec_page_refc(self) {
        pg_alloc::dec_page_refc(self);
    }
}

impl VirtAddr {
    pub unsafe fn into_slice_mut<'a>(self, size: usize) -> &'a mut [u8] {
        core::slice::from_raw_parts_mut(self.0 as *mut u8, size)
    }
}

pub trait RegisterFrameOps: fmt::Display {
    fn set_program_counter(&mut self, addr: usize);
    fn set_stack_pointer(&mut self, addr: usize);
    fn enable_interrupts(&mut self);
}

pub trait RootPageDirOps {
    fn new() -> Self;
    fn new_userspace_root_dir(info: &BootloaderInfo) -> Self;
    fn switch_to_this(&self);
    fn walk_dir(&mut self, addr: VirtAddr, create: bool) -> Option<&mut LeafDirEntry>;
    fn walk_dir_large(&mut self, addr: VirtAddr, create: bool) -> Option<&mut LeafDirEntryLarge>;
    fn map_page_at_addr(&mut self, page: &mut PageInfo, addr: VirtAddr, perms: usize);
    fn unmap_page_at_addr(&mut self, addr: VirtAddr);
    fn map_region(&mut self, from: VirtAddr, to: PhysAddr, pages: usize, perms: usize);
    fn map_region_large(&mut self, from: VirtAddr, to: PhysAddr, lpages: usize, perms: usize);
    fn unmap_region(&mut self, from: VirtAddr, pages: usize);
    fn unmap_region_large(&mut self, from: VirtAddr, lpages: usize);
    fn change_range_perms(&mut self, from: VirtAddr, size: usize, perms: usize);

    fn alloc_range(&mut self, addr: VirtAddr, size: usize, perms: usize) {
        let beg = addr.page_round_down();
        let end = (addr + size).page_round_up();

        for page_addr in (beg.0..end.0).step_by(arch::mmu::PAGE_SIZE) {
            let page = pg_alloc::alloc_page();

            self.map_page_at_addr(page, VirtAddr(page_addr), perms);
        }
    }
}
