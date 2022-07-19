// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::slice;

use crate::arch;
use crate::mm::pg_alloc;
use crate::mm::types::{Address, PhysAddr, RootPageDirOps, VirtAddr};
use crate::spinlock::Mutex;
use crate::types::{Bytes, KiB, MiB};

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

pub const PRESENT: u64 = 1 << 0;
pub const WRITABLE: u64 = 1 << 1;
pub const USER_ACCESSIBLE: u64 = 1 << 2;
pub const HUGE: u64 = 1 << 7;

static ROOT_KERN_DIR: Mutex<PageMapLevel4> = Mutex::new(PageMapLevel4::empty());

pub struct PageMapLevel4 {
    addr: u64,
    entries: &'static mut [PageMapLevel4Entry],
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
struct PageDirectoryEntry {
    scalar: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PageTableEntry {
    scalar: u64,
}

trait SetScalar {
    fn set_scalar(&mut self, val: u64);
}

trait DirectoryEntry: SetScalar + Into<u64> {
    type PointsTo;

    fn present(self) -> bool {
        self.into() & PRESENT != 0
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
        let addr = dir.to_physaddr().0 as u64;

        self.set_scalar(addr | USER_ACCESSIBLE | WRITABLE | PRESENT);
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
    fn set_scalar(&mut self, val: u64) {
        self.scalar = val;
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

impl PageMapLevel4 {
    const fn empty() -> Self {
        PageMapLevel4 {
            addr: 0,
            entries: &mut [],
        }
    }

    fn new(addr: u64) -> Self {
        let entries = unsafe {
            let addr = addr as *mut PageMapLevel4Entry;
            slice::from_raw_parts_mut(addr, ENTRIES)
        };

        Self { addr, entries }
    }

    fn clear(&mut self) {
        let zero_entry = PageMapLevel4Entry { scalar: 0 };

        self.entries.fill(zero_entry);
    }
}

pub fn init() {
    // Defined in start.s
    extern "C" {
        static pml4: u64;
    }

    *ROOT_KERN_DIR.guard().data = PageMapLevel4::new(unsafe { pml4 });
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

/// For early-stage allocation of regions, e.g. page infos and framebuffer
pub fn map_early_region(start: PhysAddr, size: usize, offset_for_virt: usize) {
    extern "C" {
        fn pd();
    }

    let num_pages = size / PAGE_SIZE_LARGE;

    println_serial!(
        "Early map {:#x}..{:#x} -> {:#x}..{:#x} ({} large page{})",
        start.0 + offset_for_virt,
        start.0 + offset_for_virt + size,
        start.0,
        start.0 + size,
        num_pages,
        if num_pages > 1 { "s" } else { "" }
    );

    let pd_ptr = pd as *mut u64;
    let range = start.0..start.0 + size;

    for phys in range.step_by(PAGE_SIZE_LARGE) {
        let virt = VirtAddr(phys + offset_for_virt);
        let frames = virt.to_2m_page_frames();

        unsafe {
            *pd_ptr.add(frames.pd_offset as usize) = phys as u64 | HUGE | WRITABLE | PRESENT;
        }
    }
}

fn walk_root_dir(addr: VirtAddr, root: &mut PageMapLevel4, create: bool) -> Option<PageTableEntry> {
    let frames = addr.to_4k_page_frames();
    let mut pml4e = root.entries[frames.pml4_offs];

    if !pml4e.present() {
        if create {
            pml4e.create_entry();
        } else {
            return None;
        }
    }

    let pdpt = pml4e.pointed_dir();
    let mut pdpe = pdpt[frames.pdpt_offs];

    if !pdpe.present() {
        if create {
            pdpe.create_entry();
        } else {
            return None;
        }
    }

    let pdt = pdpe.pointed_dir();
    let mut pde = pdt[frames.pd_offset];

    if !pde.present() {
        if create {
            pde.create_entry();
        } else {
            return None;
        }
    }

    let pt = pde.pointed_dir();
    let mut pte = pt[frames.pt_offset];

    if !pte.present() {
        if create {
            pte.create_entry();
        } else {
            return None;
        }
    }

    Some(pte)
}

pub fn is_page_present(addr: VirtAddr, root: &mut PageMapLevel4) -> bool {
    walk_root_dir(addr, root, false).is_some()
}

pub fn get_or_create_page(addr: VirtAddr, root: &mut PageMapLevel4) -> VirtAddr {
    walk_root_dir(addr, root, true).unwrap().pointed_vaddr()
}

impl RootPageDirOps for PageMapLevel4 {
    fn new() -> Self {
        let dir = pg_alloc::alloc_page().inc_refc();
        let phys = dir.to_physaddr();
        PageMapLevel4::new(phys.0 as u64)
    }

    fn switch_to_this(&self) {
        let phys = self.addr;
        write_reg!(cr3, phys);
    }

    fn map_page_at_addr(&mut self, page: &mut pg_alloc::PageInfo, addr: VirtAddr, perms: u64) {
        if let Some(mut pte) = walk_root_dir(addr, self, true) {
            page.inc_refc();

            if pte.present() {
                self.unmap_page_at_addr(addr);
            }

            let addr = page.to_physaddr().0 as u64 | perms | PRESENT;

            pte.set_scalar(addr);
        }
    }

    fn unmap_page_at_addr(&mut self, addr: VirtAddr) {
        if let Some(mut pte) = walk_root_dir(addr, self, false) {
            pte.pointed_addr().dec_page_refc();
            pte.set_scalar(pte.scalar & !PRESENT);
            arch::asm::invalidate_dcache(addr);
        }
    }
}
