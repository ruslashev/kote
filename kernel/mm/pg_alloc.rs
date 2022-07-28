// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut, NonNull};

use crate::arch::{mmu, KERNEL_BASE};
use crate::bootloader::{BootloaderInfo, Region, SectionInfoIterator};
use crate::mm::types::{Address, PhysAddr, VirtAddr};
use crate::spinlock::Mutex;
use crate::types::PowerOfTwoOps;

static PAGE_INFOS: Mutex<&mut [PageInfo]> = Mutex::new(&mut []);
static FREE_PAGES: Mutex<Option<NonNull<PageInfo>>> = Mutex::new(None);

#[derive(Default)]
pub struct PageInfo {
    next: Option<NonNull<PageInfo>>,
    refc: u16,
}

impl PageInfo {
    fn alloc() -> Option<&'static mut PageInfo> {
        let mut freep = FREE_PAGES.lock();
        let mut head = (*freep)?;
        let pgref = unsafe { head.as_mut() };

        unsafe {
            let vaddr = pgref.to_physaddr().into_vaddr();
            let region = vaddr.into_slice_mut(mmu::PAGE_SIZE);
            region.fill(0);
        }

        *freep = pgref.next;

        pgref.next = None;

        Some(pgref)
    }

    fn free(&mut self) {
        assert!(self.refc == 0, "free_page: page is used");
        let mut freep = FREE_PAGES.lock();

        self.next = *freep;

        *freep = unsafe { Some(NonNull::new_unchecked(self as *mut _)) };
    }

    pub fn to_physaddr(&self) -> PhysAddr {
        let infos = PAGE_INFOS.lock();
        let base = addr_of!(infos[0]) as usize;
        let this = addr_of!(*self) as usize;
        let offset = this - base;
        let index = offset / size_of::<PageInfo>();
        let addr = index * mmu::PAGE_SIZE;

        PhysAddr(addr)
    }

    pub fn inc_refc(&mut self) -> &mut Self {
        self.refc += 1;
        self
    }

    pub fn dec_refc(&mut self) -> &mut Self {
        self.refc -= 1;
        if self.refc == 0 {
            self.free();
        }
        self
    }
}

pub fn dec_page_refc(addr: PhysAddr) {
    let idx = addr.0 / mmu::PAGE_SIZE;
    let mut infos = PAGE_INFOS.lock();
    infos[idx].dec_refc();
}

pub fn init(area_start: VirtAddr, maxpages: usize, info: &mut BootloaderInfo) {
    let mut infos = PAGE_INFOS.lock();
    let mut freep = FREE_PAGES.lock();

    *infos = unsafe { core::slice::from_raw_parts_mut(area_start.0 as *mut PageInfo, maxpages) };
    infos.fill_with(Default::default); // mark all as non-free

    *freep = None;

    println_serial!(
        "Initializing page information list at {:x?}..{:x?}...",
        area_start,
        area_start + maxpages * size_of::<PageInfo>()
    );

    let mmap = &info.free_areas;
    for eidx in (0..mmap.num_entries).rev() {
        let Region { start, end } = mmap.entries[eidx];

        if end - start < mmu::PAGE_SIZE {
            continue;
        }

        let pg_start = start.page_round_up();
        let pg_end = end.page_round_down();

        println_serial!("Free area {:x?}..{:x?}", pg_start, pg_end);

        for pg in (pg_start..pg_end).step_by(mmu::PAGE_SIZE).rev() {
            let index = pg / mmu::PAGE_SIZE;

            if infos[index].next.is_some() {
                continue;
            }

            infos[index].next = *freep;
            *freep = unsafe { Some(NonNull::new_unchecked(addr_of_mut!(infos[index]))) };
        }
    }
}

pub fn get_pg_alloc_region(info: &BootloaderInfo) -> (usize, PhysAddr, usize) {
    let alloc_start = get_kernel_end(info).lpage_round_up();

    let mmap = &info.free_areas;
    let max_addr = mmap.entries[mmap.num_entries - 1].end;
    let maxpages = max_addr.div_ceil(mmu::PAGE_SIZE);
    let size_bytes = maxpages * size_of::<PageInfo>();
    let size_rounded = size_bytes.lpage_round_up();

    (maxpages, PhysAddr::from_u64(alloc_start), size_rounded)
}

fn get_kernel_end(info: &BootloaderInfo) -> u64 {
    let mut kernel_end = 0;

    for (_, &shdr) in SectionInfoIterator::from_info(info.section_headers.as_ref().unwrap()) {
        if shdr.sh_addr != 0 {
            let mut end = shdr.sh_addr + shdr.sh_size;
            if end > KERNEL_BASE {
                end -= KERNEL_BASE;
            }
            if end > kernel_end {
                kernel_end = end;
            }
        }
    }

    kernel_end
}

pub fn alloc_page() -> &'static mut PageInfo {
    // This should not die on OOM
    PageInfo::alloc().expect("pg_alloc: out of memory")
}
