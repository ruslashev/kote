// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::mm::addr::{Address, PhysAddr, VirtAddr};
use crate::spinlock::SpinlockMutex;

pub const PAGE_SIZE: u64 = 4096;
pub const PAGE_SIZE_LARGE: u64 = 2 * 1024 * 1024; // 2 MiB

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

static ROOT_DIR: SpinlockMutex<PML4> = SpinlockMutex::new(PML4::empty());

struct PML4 {
    addr: u64,
    entries: &'static mut [PML4E],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PML4E {
    scalar: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PDPE {
    scalar: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PDE {
    scalar: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PTE {
    scalar: u64,
}

impl PML4 {
    const fn empty() -> Self {
        PML4 {
            addr: 0,
            entries: &mut [],
        }
    }

    fn from_addr(addr: u64) -> Self {
        let entries = unsafe {
            let addr = addr as *mut PML4E;
            core::slice::from_raw_parts_mut(addr, ENTRIES)
        };

        Self { addr, entries }
    }

    fn clear(&mut self) {
        let zero_entry = PML4E { scalar: 0 };

        self.entries.fill(zero_entry);
    }
}

pub fn init() {
    // Defined in start.s
    extern "C" {
        static pml4: u64;
    }

    *ROOT_DIR.guard().data = PML4::from_addr(unsafe { pml4 });
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

trait ToFrames: Address {
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

pub fn map(from: VirtAddr, to: PhysAddr) {
    println!("map {} {}", from, to);
    println!("{:?}", from.to_2m_page_frames());
}
