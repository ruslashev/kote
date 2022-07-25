// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod pg_alloc;
pub mod types;

use self::types::{Address, PhysAddr, VirtAddr};
use crate::arch::{self, mmu};
use crate::bootloader::BootloaderInfo;
use crate::types::PowerOfTwoOps;

pub fn init(info: &mut BootloaderInfo) -> VirtAddr {
    mmu::init();

    let (pg_alloc_start, fb_start, maxpages) = prepare_page_alloc_region(info);

    pg_alloc::init(pg_alloc_start, maxpages, info);

    fb_start
}

fn prepare_page_alloc_region(info: &mut BootloaderInfo) -> (VirtAddr, VirtAddr, usize) {
    let (maxpages, start, size) = pg_alloc::get_pg_alloc_region(info);
    let start_vaddr = start.into_vaddr();
    let end = start_vaddr + size;

    mmu::map_early_region(start, size, arch::KERNEL_BASE as usize);
    info.free_areas.remove_range(start, size);

    (start_vaddr, end, maxpages)
}

fn map_framebuffer(start: VirtAddr, info: &mut BootloaderInfo) {
    let fb = &info.framebuffer;
    let phys = PhysAddr::from_u64(fb.addr);
    let size = fb.pitch * fb.height;
    let size = usize::try_from(size).expect("framebuffer size overflows usize");
    let size = size.lpage_round_up();
    let virt = start;
    let offset = virt.0 - phys.0;

    mmu::map_early_region(phys, size, offset);

    info.free_areas.remove_range(phys, size);
}
