// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::mem::size_of;

use crate::arch::mmu;
use crate::arch::mmu::{PAGE_SIZE, PAGE_SIZE_LARGE};
use crate::arch::KERNEL_BASE;
use crate::bootloader::{BootloaderInfo, SectionInfoIterator};
use crate::mm::addr::{Address, PhysAddr, VirtAddr};
use crate::units::po2_round_up;

#[repr(packed)]
struct PageInfo<'a> {
    next: Option<&'a PageInfo<'a>>,
    refc: u16,
}

pub fn init(info: &BootloaderInfo) {
    let (start, size) = get_page_infos_region(info);

    println!("start={:#x}, size={:#x}", start, size);

    let num_largepages = size / PAGE_SIZE_LARGE;
    let mut page_start = start;

    for _ in 0..num_largepages {
        let phys = page_start;
        let virt = phys + KERNEL_BASE;

        mmu::map(VirtAddr::from_u64(virt), PhysAddr::from_u64(phys));

        page_start += PAGE_SIZE_LARGE;
    }
}

fn get_page_infos_region(info: &BootloaderInfo) -> (u64, u64) {
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

    let alloc_start = po2_round_up(kernel_end, PAGE_SIZE_LARGE);

    let mmap = &info.free_areas;
    let max_addr = mmap.entries[mmap.num_entries - 1].end;
    let maxpages = max_addr.div_ceil(PAGE_SIZE as usize);
    let page_infos_bytes = maxpages * size_of::<PageInfo>();
    let page_infos_rounded = po2_round_up(page_infos_bytes as u64, PAGE_SIZE_LARGE);

    (alloc_start, page_infos_rounded)
}
