// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod pg_alloc;
pub mod types;

use self::types::{PhysAddr, VirtAddr};
use crate::arch::{self, mmu, RootPageDir};
use crate::bootloader::BootloaderInfo;
use crate::mm::types::RootPageDirOps;
use crate::spinlock::Mutex;
use crate::types::PowerOfTwoOps;

static ROOT_KERN_DIR: Mutex<RootPageDir> = Mutex::new(arch::EMPTY_ROOT_DIR);

pub fn init(info: &mut BootloaderInfo) {
    let (maxpages, pg_alloc_start, pg_alloc_size) = pg_alloc::get_pg_alloc_region(info);

    info.free_areas.remove_range(pg_alloc_start, pg_alloc_size);

    pg_alloc::init(pg_alloc_start.into_vaddr(), maxpages, info);

    *ROOT_KERN_DIR.lock() = create_kern_root_dir(maxpages);
}

fn create_kern_root_dir(maxpages: usize) -> RootPageDir {
    let mut root_dir = RootPageDir::new();
    let phys_flags = mmu::PRESENT | mmu::WRITABLE;

    println_serial!("Mapping physical memory...");

    let phys_size = maxpages * mmu::PAGE_SIZE;
    let lpages = phys_size.div_ceil(mmu::PAGE_SIZE_LARGE);
    root_dir.map_region_large(VirtAddr(arch::KERNEL_BASE), PhysAddr(0), lpages, phys_flags);

    println_serial!("Mapping stack guards...");

    extern "C" {
        fn stack_guard_top();
        fn stack_guard_bot();
    }
    let top = VirtAddr(stack_guard_top as usize);
    let bot = VirtAddr(stack_guard_bot as usize);
    let top_large = top.lpage_round_down();
    let bot_large = bot.lpage_round_down();

    // Memory on guard pages was covered by a large-page mapping above. Unmap it first.
    root_dir.unmap_region_large(top_large, 1);
    root_dir.unmap_region_large(bot_large, 1);

    // Recreate the mapping but with lower granularity
    root_dir.map_region(top_large, PhysAddr(0), mmu::PAGE_SIZE_LARGE / mmu::PAGE_SIZE, phys_flags);
    root_dir.map_region(bot_large, PhysAddr(0), mmu::PAGE_SIZE_LARGE / mmu::PAGE_SIZE, phys_flags);

    // Finally, unmap guard pages
    root_dir.unmap_region(top, 1);
    root_dir.unmap_region(bot, 1);

    root_dir.switch_to_this();

    root_dir
}

pub fn switch_to_kernel_root_dir() {
    ROOT_KERN_DIR.lock().switch_to_this();
}
