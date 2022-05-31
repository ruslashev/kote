// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::mem::size_of;

use crate::arch::mmu;
use crate::arch::KERNEL_BASE;
use crate::bootloader::{BootloaderInfo, Region, SectionInfoIterator};
use crate::types::PowerOfTwoOps;

static mut PAGE_INFOS: &mut [PageInfo] = &mut [];
static mut FREE_PAGES: Option<&PageInfo> = None;
static mut FREE_PAGES_END: Option<&PageInfo> = None;

#[derive(Default)]
struct PageInfo<'a> {
    next: Option<&'a PageInfo<'a>>,
    refc: u16,
}

pub fn init(info: &BootloaderInfo) -> u64 {
    init_page_infos(info)
}

fn init_page_infos(info: &BootloaderInfo) -> u64 {
    let (maxpages, start, size) = get_page_infos_region(info);
    let end = KERNEL_BASE + start + size;

    mmu::map_early_region(start, size, KERNEL_BASE);

    unsafe {
        PAGE_INFOS = core::slice::from_raw_parts_mut(start as *mut PageInfo, maxpages);
        PAGE_INFOS.fill_with(Default::default); // mark all as non-free

        FREE_PAGES = None;
        FREE_PAGES_END = Some(&PAGE_INFOS[1]);
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
                FREE_PAGES = Some(&PAGE_INFOS[index]);
            }
        }
    }

    end
}

fn get_page_infos_region(info: &BootloaderInfo) -> (usize, u64, u64) {
    let kernel_end = get_kernel_end(info);
    let alloc_start = kernel_end.lpage_round_up();

    let mmap = &info.free_areas;
    let max_addr = mmap.entries[mmap.num_entries - 1].end;
    let maxpages = max_addr.div_ceil(mmu::PAGE_SIZE);
    let page_infos_bytes = maxpages * size_of::<PageInfo>();
    let page_infos_rounded = (page_infos_bytes as u64).lpage_round_up();

    (maxpages, alloc_start, page_infos_rounded)
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
