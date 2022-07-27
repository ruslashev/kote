// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod pg_alloc;
pub mod types;

use self::types::{Address, PhysAddr, VirtAddr};
use crate::arch::{self, mmu, RootPageDir};
use crate::bootloader::BootloaderInfo;
use crate::mm::types::RootPageDirOps;
use crate::spinlock::Mutex;

static ROOT_KERN_DIR: Mutex<RootPageDir> = Mutex::new(arch::EMPTY_ROOT_DIR);

pub fn init(info: &mut BootloaderInfo) {
    let (pg_alloc_start, maxpages) = prepare_page_alloc_region(info);

    pg_alloc::init(pg_alloc_start, maxpages, info);

    let mut kern_root_dir = ROOT_KERN_DIR.guard();
    *kern_root_dir = create_kern_root_dir(maxpages);
    kern_root_dir.switch_to_this();
}

fn prepare_page_alloc_region(info: &mut BootloaderInfo) -> (VirtAddr, usize) {
    let (maxpages, start, size) = pg_alloc::get_pg_alloc_region(info);

    mmu::map_pg_alloc_region(start, size, arch::KERNEL_BASE as usize);
    info.free_areas.remove_range(start, size);

    (start.into_vaddr(), maxpages)
}

fn create_kern_root_dir(maxpages: usize) -> RootPageDir {
    let mut root_dir = RootPageDir::new();

    println_serial!("Mapping physical memory...");
    let phys_size = maxpages * mmu::PAGE_SIZE;
    let lpages = phys_size.div_ceil(mmu::PAGE_SIZE_LARGE);
    root_dir.map_static_region(
        VirtAddr::from_u64(arch::KERNEL_BASE),
        PhysAddr(0),
        lpages,
        mmu::WRITABLE,
    );

    root_dir.map_static_region(VirtAddr(0), PhysAddr(0), 64, 0);

    root_dir
}
