// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::spinlock::SpinlockMutex;
use crate::types::{Address, Bytes, KiB, MiB, VirtAddr};

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

impl PageMapLevel4 {
    const fn empty() -> Self {
        PageMapLevel4 {
            addr: 0,
            entries: &mut [],
        }
    }

    fn from_addr(addr: u64) -> Self {
        let entries = unsafe {
            let addr = addr as *mut PageMapLevel4Entry;
            core::slice::from_raw_parts_mut(addr, ENTRIES)
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

    *ROOT_DIR.guard().data = PageMapLevel4::from_addr(unsafe { pml4 });
}

#[derive(Debug)]
pub struct PageFrames4K {
    pml4_o: u64,
    pdpt_o: u64,
    pd_off: u64,
    pt_off: u64,
    offset: u64,
}

#[derive(Debug)]
pub struct PageFrames2M {
    pml4_o: u64,
    pdpt_o: u64,
    pd_off: u64,
    offset: u64,
}

trait ToFrames {
    fn to_4k_page_frames(&self) -> PageFrames4K;
    fn to_2m_page_frames(&self) -> PageFrames2M;
}

impl ToFrames for VirtAddr {
    fn to_4k_page_frames(&self) -> PageFrames4K {
        let addr: u64 = self.0.try_into().unwrap();

        PageFrames4K {
            pml4_o: (addr & 0xff8000000000) >> 39,
            pdpt_o: (addr & 0x007fc0000000) >> 30,
            pd_off: (addr & 0x00003fe00000) >> 21,
            pt_off: (addr & 0x0000001ff000) >> 12,
            offset: (addr & 0x000000000fff) >> 0,
        }
    }

    fn to_2m_page_frames(&self) -> PageFrames2M {
        let addr: u64 = self.0.try_into().unwrap();

        PageFrames2M {
            pml4_o: (addr & 0xff8000000000) >> 39,
            pdpt_o: (addr & 0x007fc0000000) >> 30,
            pd_off: (addr & 0x00003fe00000) >> 21,
            offset: (addr & 0x0000001fffff) >> 0,
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
            *pd_ptr.add(frames.pd_off as usize) = phys | HUGE | WRITABLE | PRESENT;
        }
    }
}
