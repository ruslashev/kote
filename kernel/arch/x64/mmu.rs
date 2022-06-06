// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::slice;

use crate::mm::pg_alloc;
use crate::spinlock::SpinlockMutex;
use crate::types::{Address, Bytes, KiB, MiB, PhysAddr, VirtAddr};

pub const PAGE_SIZE: usize = KiB(4).to_bytes();
pub const PAGE_SIZE_LARGE: usize = MiB(2).to_bytes();

/* Memory layout:
 * ┌───────────────────────────────┐ 0xffffffffffffffff
 * │                               │
 * │                               │
 * │  Identity mapping for kernel  │ 0xffffffff80000000 KERNEL_BASE
 * ├───────────────────────────────┤
 * │   Page allocation structures  │
 * ├───────────────────────────────┤
 * │      Framebuffer mapping      │
 * ├───────────────────────────────┤
 * │                               │
 * │          Kernel heap          │
 *
 * ╵               .               ╵
 * ╵               .               ╵
 *
 * │                               │
 * └───────────────────────────────┘
 */

/// Number of entries in a directory of any level (PML4, PDPT, PD, PT). Equal to 4096 B / 64 b.
const ENTRIES: usize = 512;

const PRESENT: u64 = 1 << 0;
const WRITABLE: u64 = 1 << 1;
const USER_ACCESSIBLE: u64 = 1 << 2;
const HUGE: u64 = 1 << 7;

static ROOT_DIR: SpinlockMutex<PageMapLevel4> = SpinlockMutex::new(PageMapLevel4::empty());

struct PageMapLevel4 {
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
    fn pointed_addr(self) -> VirtAddr {
        let paddr = self.into() & 0xffffffffff000;
        PhysAddr::from(paddr as usize).into_vaddr()
    }

    fn pointed_dir<'a>(self) -> &'a mut [Self::PointsTo] {
        let vaddr = self.pointed_addr();
        let ptr = vaddr.0 as *mut Self::PointsTo;

        unsafe { slice::from_raw_parts_mut(ptr, ENTRIES) }
    }

    unsafe fn create_entry(&mut self) {
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

    *ROOT_DIR.guard().data = PageMapLevel4::new(unsafe { pml4 });
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
pub fn map_early_region(start: u64, size: u64, offset_for_virt: u64) {
    extern "C" {
        fn pd();
    }

    println_serial!(
        "Early map {:#x}..{:#x} -> {:#x}..{:#x} ({} large pages)",
        offset_for_virt + start,
        offset_for_virt + start + size,
        start,
        start + size,
        size / PAGE_SIZE_LARGE as u64
    );

    let pd_ptr = pd as *mut u64;

    for phys in (start..start + size).step_by(PAGE_SIZE_LARGE) {
        let virt = phys + offset_for_virt;
        let virt = VirtAddr::from_u64(virt);
        let frames = virt.to_2m_page_frames();

        unsafe {
            *pd_ptr.add(frames.pd_offset as usize) = phys | HUGE | WRITABLE | PRESENT;
        }
    }
}

unsafe fn walk_root_dir(addr: VirtAddr, create: bool) -> Option<VirtAddr> {
    let frames = addr.to_4k_page_frames();
    let root = ROOT_DIR.guard();
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

    Some(pte.pointed_addr())
}

pub fn is_page_present(addr: VirtAddr) -> bool {
    unsafe { walk_root_dir(addr, false).is_some() }
}

pub fn get_or_create_page(addr: VirtAddr) -> VirtAddr {
    unsafe { walk_root_dir(addr, true).unwrap() }
}
