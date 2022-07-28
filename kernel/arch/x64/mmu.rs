// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::slice;

use crate::arch::{self, LeafDirEntry, LeafDirEntryLarge};
use crate::mm::pg_alloc;
use crate::mm::types::{Address, PhysAddr, RootPageDirOps, VirtAddr};
use crate::types::{Bytes, KiB, MiB, PowerOfTwoOps};

pub const PAGE_SIZE: usize = KiB(4).to_bytes();
pub const PAGE_SIZE_LARGE: usize = MiB(2).to_bytes();

/* Memory layout:
 * ┌───────────────────────────────┐ 0xffffffffffffffff
 * │                               │
 * │                               │
 * │  Identity mapping for kernel  │ 0xffffff8000000000 KERNEL_BASE
 * ├───────────────────────────────┤
 * │                               │
 * │             TODO              │
 *
 * ╵               .               ╵
 * ╵               .               ╵
 *
 * │                               │
 * └───────────────────────────────┘
 */

/// Number of entries in a directory of any level (PML4, PDPT, PD, PT). Equal to 4096 B / 64 b.
const ENTRIES: usize = 512;

pub const PRESENT: usize = 1 << 0;
pub const WRITABLE: usize = 1 << 1;
pub const USER_ACCESSIBLE: usize = 1 << 2;
pub const HUGE: usize = 1 << 7;

pub struct PageMapLevel4 {
    addr: usize,
}

impl PageMapLevel4 {
    pub const fn empty() -> Self {
        Self { addr: 0 }
    }

    fn as_slice<'a>(&self) -> &'a [PageMapLevel4Entry] {
        unsafe {
            let ptr = self.addr as *const PageMapLevel4Entry;
            slice::from_raw_parts(ptr, ENTRIES)
        }
    }

    fn as_slice_mut<'a>(&mut self) -> &'a mut [PageMapLevel4Entry] {
        unsafe {
            let ptr = self.addr as *mut PageMapLevel4Entry;
            slice::from_raw_parts_mut(ptr, ENTRIES)
        }
    }
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PageMapLevel4Entry {
    scalar: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PageDirectoryPointerEntry {
    scalar: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct PageDirectoryEntry {
    scalar: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
pub struct PageTableEntry {
    scalar: u64,
}

trait SetScalar {
    fn set_scalar(&mut self, val: usize);
}

trait DirectoryEntry: SetScalar + Into<u64> {
    type PointsTo;

    fn present(self) -> bool {
        self.into() & PRESENT as u64 != 0
    }

    /// Get the address of directory this entry points to
    fn pointed_addr(self) -> PhysAddr {
        let paddr = self.into() & 0xffffffffff000;
        PhysAddr::from_u64(paddr)
    }

    fn pointed_vaddr(self) -> VirtAddr {
        self.pointed_addr().into_vaddr()
    }

    fn pointed_dir<'a>(self) -> &'a mut [Self::PointsTo] {
        let vaddr = self.pointed_vaddr();
        let ptr = vaddr.0 as *mut Self::PointsTo;

        unsafe { slice::from_raw_parts_mut(ptr, ENTRIES) }
    }

    fn create_entry(&mut self) {
        let dir = pg_alloc::alloc_page().inc_refc();
        let addr = dir.to_physaddr().0 | WRITABLE | PRESENT;

        self.set_scalar(addr);
    }
}

macro_rules! impl_directory_traits {
    ( $( $type:ident )+ ) => {
        $(
impl From<$type> for u64 {
    fn from(val: $type) -> Self {
        val.scalar
    }
}
impl SetScalar for $type {
    fn set_scalar(&mut self, val: usize) {
        self.scalar = val as u64;
    }
}
        )*
    }
}

impl_directory_traits!(PageMapLevel4Entry PageDirectoryPointerEntry PageDirectoryEntry PageTableEntry);

impl DirectoryEntry for PageMapLevel4Entry {
    type PointsTo = PageDirectoryPointerEntry;
}

impl DirectoryEntry for PageDirectoryPointerEntry {
    type PointsTo = PageDirectoryEntry;
}

impl DirectoryEntry for PageDirectoryEntry {
    type PointsTo = PageTableEntry;
}

impl DirectoryEntry for PageTableEntry {
    type PointsTo = u64;
}

#[derive(Debug)]
pub struct PageFrames4K {
    pml4_offs: usize,
    pdpt_offs: usize,
    pd_offset: usize,
    pt_offset: usize,
    pg_offset: usize,
}

#[derive(Debug)]
pub struct PageFrames2M {
    pml4_offs: usize,
    pdpt_offs: usize,
    pd_offset: usize,
    pg_offset: usize,
}

trait ToFrames {
    fn to_4k_page_frames(&self) -> PageFrames4K;
    fn to_2m_page_frames(&self) -> PageFrames2M;
}

impl ToFrames for VirtAddr {
    fn to_4k_page_frames(&self) -> PageFrames4K {
        PageFrames4K {
            pml4_offs: (self.0 & 0xff8000000000) >> 39,
            pdpt_offs: (self.0 & 0x007fc0000000) >> 30,
            pd_offset: (self.0 & 0x00003fe00000) >> 21,
            pt_offset: (self.0 & 0x0000001ff000) >> 12,
            pg_offset: (self.0 & 0x000000000fff) >> 0,
        }
    }

    fn to_2m_page_frames(&self) -> PageFrames2M {
        PageFrames2M {
            pml4_offs: (self.0 & 0xff8000000000) >> 39,
            pdpt_offs: (self.0 & 0x007fc0000000) >> 30,
            pd_offset: (self.0 & 0x00003fe00000) >> 21,
            pg_offset: (self.0 & 0x0000001fffff) >> 0,
        }
    }
}

impl RootPageDirOps for PageMapLevel4 {
    fn new() -> Self {
        let dir = pg_alloc::alloc_page().inc_refc();
        let phys = dir.to_physaddr();
        PageMapLevel4 { addr: phys.0 }
    }

    fn switch_to_this(&self) {
        let phys = self.addr as u64;
        write_reg!(cr3, phys);
    }

    fn walk_dir(&mut self, addr: VirtAddr, create: bool) -> Option<&mut LeafDirEntry> {
        let frames = addr.to_4k_page_frames();
        let pml4e = &mut self.as_slice_mut()[frames.pml4_offs];

        if !pml4e.present() {
            if create {
                pml4e.create_entry();
            } else {
                return None;
            }
        }

        let pdpt = pml4e.pointed_dir();
        let pdpe = &mut pdpt[frames.pdpt_offs];

        if !pdpe.present() {
            if create {
                pdpe.create_entry();
            } else {
                return None;
            }
        }

        let pdt = pdpe.pointed_dir();
        let pde = &mut pdt[frames.pd_offset];

        if !pde.present() {
            if create {
                pde.create_entry();
            } else {
                return None;
            }
        }

        let pt = pde.pointed_dir();
        let pte = &mut pt[frames.pt_offset];

        if !pte.present() {
            if create {
                pte.create_entry();
            } else {
                return None;
            }
        }

        Some(pte)
    }

    fn walk_dir_large(&mut self, addr: VirtAddr, create: bool) -> Option<&mut LeafDirEntryLarge> {
        let frames = addr.to_2m_page_frames();
        let pml4e = &mut self.as_slice_mut()[frames.pml4_offs];

        if !pml4e.present() {
            if create {
                pml4e.create_entry();
            } else {
                return None;
            }
        }

        let pdpt = pml4e.pointed_dir();
        let pdpe = &mut pdpt[frames.pdpt_offs];

        if !pdpe.present() {
            if create {
                pdpe.create_entry();
            } else {
                return None;
            }
        }

        let pdt = pdpe.pointed_dir();
        let pde = &mut pdt[frames.pd_offset];

        if !pde.present() {
            if create {
                pde.create_entry();
            } else {
                return None;
            }
        }

        Some(pde)
    }

    fn map_page_at_addr(&mut self, page: &mut pg_alloc::PageInfo, addr: VirtAddr, perms: usize) {
        let pte = self.walk_dir(addr, true).unwrap();
        page.inc_refc();

        if pte.present() {
            pte.pointed_addr().dec_page_refc();
            pte.set_scalar(pte.scalar as usize & !PRESENT);
            arch::asm::invalidate_dcache(addr);
        }

        let addr = page.to_physaddr().0 | perms | PRESENT;

        pte.set_scalar(addr);
    }

    fn unmap_page_at_addr(&mut self, addr: VirtAddr) {
        if let Some(pte) = self.walk_dir(addr, false) {
            pte.pointed_addr().dec_page_refc();
            pte.set_scalar(pte.scalar as usize & !PRESENT);
            arch::asm::invalidate_dcache(addr);
        }
    }

    fn map_static_region(&mut self, from: VirtAddr, to: PhysAddr, lpages: usize, perms: usize) {
        assert!(from.0.is_lpage_aligned());
        assert!(to.0.is_lpage_aligned());

        let size = lpages * PAGE_SIZE_LARGE;

        println_serial!(
            "Map {:#x}..{:#x} -> {:#x}..{:#x} ({} large page{}, {} MiB)",
            from.0,
            from.0 + size,
            to.0,
            to.0 + size,
            lpages,
            if lpages > 1 { "s" } else { "" },
            size / 1024 / 1024
        );

        for page in 0..lpages {
            let vaddr = from + page * PAGE_SIZE_LARGE;
            let pde = self.walk_dir_large(vaddr, true).unwrap();
            let addr = to.0 + page * PAGE_SIZE_LARGE;

            pde.set_scalar(addr | perms | PRESENT | HUGE);
        }
    }
}

pub fn prepare_userspace_root_dir(_root_dir: &mut PageMapLevel4) {
    // Temporary
    // root_dir.as_slice_mut().copy_from_slice(ROOT_KERN_DIR.guard().as_slice());
}
