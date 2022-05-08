// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod addr;
mod pg_alloc;

use crate::arch;
use crate::bootloader::BootloaderInfo;
use crate::units::po2_round_up;

pub fn init(info: &BootloaderInfo) -> usize {
    arch::mmu::init();
    let page_infos_end = pg_alloc::init(info);

    map_framebuffer(page_infos_end, info);

    usize::try_from(page_infos_end).expect("framebuffer address overflows usize")
}

fn map_framebuffer(page_infos_end: u64, info: &BootloaderInfo) {
    let fb = &info.framebuffer;
    let phys = fb.addr;
    let size = fb.pitch * fb.height;
    let size = po2_round_up(size.into(), arch::mmu::PAGE_SIZE_LARGE);
    let virt = page_infos_end;
    let offset = virt - phys;

    arch::mmu::map_early_region(phys, size, offset);
}
