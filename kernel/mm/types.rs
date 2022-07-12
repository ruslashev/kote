// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;

use crate::arch::KERNEL_BASE;
use crate::mm::pg_alloc::PageInfo;

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
        write!(f, "{:x?}", &self)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x?}", &self)
    }
}

impl From<PhysAddr> for VirtAddr {
    fn from(paddr: PhysAddr) -> Self {
        VirtAddr(paddr.0 + KERNEL_BASE as usize)
    }
}

impl PhysAddr {
    pub fn into_vaddr(self) -> VirtAddr {
        self.into()
    }

    pub fn into_page(self) -> &'static mut PageInfo {
        self.into()
    }
}

impl VirtAddr {
    pub unsafe fn into_slice_mut<'a>(self, size: usize) -> &'a mut [u8] {
        core::slice::from_raw_parts_mut(self.0 as *mut u8, size)
    }
}

pub trait RegisterFrameOps: fmt::Display {
    fn set_program_counter(&mut self, addr: usize);
}

pub trait RootPageDirOps {
    fn new() -> Self;
    fn switch_to_this(&self);
    unsafe fn map_page_at_addr(&mut self, page: &mut PageInfo, addr: VirtAddr, perms: u64);
    unsafe fn unmap_page_at_addr(&mut self, addr: VirtAddr);
}
