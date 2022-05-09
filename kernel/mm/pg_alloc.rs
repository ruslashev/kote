// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::mem::size_of;

use crate::arch::mmu;
use crate::arch::KERNEL_BASE;
use crate::bootloader::{BootloaderInfo, SectionInfoIterator};
use crate::types::PowerOfTwoOps;

#[repr(packed)]
struct PageInfo<'a> {
    next: Option<&'a PageInfo<'a>>,
    refc: u16,
}

pub fn init(info: &BootloaderInfo) -> u64 {
    let (start, size) = get_page_infos_region(info);
    let end = KERNEL_BASE + start + size;

    mmu::map_early_region(start, size, KERNEL_BASE);

    end
}

fn get_page_infos_region(info: &BootloaderInfo) -> (u64, u64) {
    let kernel_end = get_kernel_end(info);
    let alloc_start = kernel_end.lpage_round_up();

    let mmap = &info.free_areas;
    let max_addr = mmap.entries[mmap.num_entries - 1].end;
    let maxpages = max_addr.div_ceil(mmu::PAGE_SIZE);
    let page_infos_bytes = maxpages * size_of::<PageInfo>();
    let page_infos_rounded = (page_infos_bytes as u64).lpage_round_up();

    (alloc_start, page_infos_rounded)
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
