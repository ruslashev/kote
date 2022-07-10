// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::mem::size_of;
use core::ptr::{addr_of, addr_of_mut, NonNull};

use crate::arch::{mmu, KERNEL_BASE};
use crate::bootloader::{BootloaderInfo, Region, SectionInfoIterator};
use crate::mm::types::{Address, PhysAddr, VirtAddr};
use crate::types::PowerOfTwoOps;

static mut PAGE_INFOS: &mut [PageInfo] = &mut [];
static mut FREE_PAGES: Option<NonNull<PageInfo>> = None;

#[derive(Default)]
pub struct PageInfo {
    next: Option<NonNull<PageInfo>>,
    refc: u16,
}

impl PageInfo {
    unsafe fn alloc() -> Option<&'static mut PageInfo> {
        match FREE_PAGES {
            Some(mut head) => {
                let pgref = head.as_mut();
                let vaddr = pgref.to_physaddr().into_vaddr();
                let region = vaddr.into_slice_mut(mmu::PAGE_SIZE);

                region.fill(0);

                FREE_PAGES = pgref.next;

                pgref.next = None;

                Some(pgref)
            }
            None => None,
        }
    }

    unsafe fn free(&mut self) {
        assert!(self.refc == 0, "free_page: page is used");

        self.next = FREE_PAGES;

        FREE_PAGES = Some(NonNull::new_unchecked(self as *mut _));
    }

    pub unsafe fn to_physaddr(&self) -> PhysAddr {
        let base = addr_of!(PAGE_INFOS) as usize;
        let this = addr_of!(self) as usize;
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
            unsafe {
                self.free();
            }
        }
        self
    }
}

impl From<PhysAddr> for &mut PageInfo {
    fn from(paddr: PhysAddr) -> Self {
        unsafe {
            let idx = paddr.0 / mmu::PAGE_SIZE;
            &mut PAGE_INFOS[idx]
        }
    }
}

pub fn init(area_start: VirtAddr, maxpages: usize, info: &mut BootloaderInfo) {
    unsafe {
        PAGE_INFOS = core::slice::from_raw_parts_mut(area_start.0 as *mut PageInfo, maxpages);
        PAGE_INFOS.fill_with(Default::default); // mark all as non-free

        FREE_PAGES = None;
    }

    println_serial!("Initializing page information list...");

    let mmap = &info.free_areas;
    for eidx in 0..mmap.num_entries {
        let Region { start, end } = mmap.entries[eidx];
        let pg_start = start.page_round_down();
        let pg_end = end.page_round_up();

        for pg in (pg_start..pg_end).into_iter().step_by(mmu::PAGE_SIZE) {
            let index = pg / mmu::PAGE_SIZE;

            unsafe {
                if PAGE_INFOS[index].next.is_some() {
                    continue;
                }

                PAGE_INFOS[index].next = FREE_PAGES;
                FREE_PAGES = Some(NonNull::new_unchecked(addr_of_mut!(PAGE_INFOS[index])));
            }
        }
    }
}

pub fn get_pg_alloc_region(info: &BootloaderInfo) -> (usize, PhysAddr, usize) {
    let alloc_start = get_kernel_end(info).lpage_round_up();

    let mmap = &info.free_areas;
    let max_addr = mmap.entries[mmap.num_entries - 1].end;
    let maxpages = max_addr.div_ceil(mmu::PAGE_SIZE);
    let page_infos_bytes = maxpages * size_of::<PageInfo>();
    let page_infos_rounded = page_infos_bytes.lpage_round_up();

    (maxpages, PhysAddr::from_u64(alloc_start), page_infos_rounded)
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
    unsafe { PageInfo::alloc().expect("pg_alloc: out of memory") }
}
