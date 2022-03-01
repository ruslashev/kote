// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod multiboot;

use crate::elf::Elf64Shdr;

const MMAP_MAX_ENTRIES: usize = 10;

trait Bootloader {
    fn get_info() -> BootloaderInfo;
}

#[derive(Default)]
pub struct BootloaderInfo {
    pub memory_map: Option<MemoryMap>,
    pub framebuffer: FramebufferInfo,
    pub section_headers: Option<SectionInfo>,
}

#[derive(Default)]
pub struct MemoryMap {
    pub regions: [Region; MMAP_MAX_ENTRIES],
    pub num_entries: usize,
}

#[derive(Default, Clone, Copy)]
pub struct Region {
    pub start: usize,
    pub end: usize,
}

#[derive(Default)]
pub struct FramebufferInfo {
    pub addr: u64,
    pub width: u32,
    pub height: u32,
    pub pitch: u32,
    pub bpp: u8,
    pub red_pos: u8,
    pub red_mask_sz: u8,
    pub green_pos: u8,
    pub green_mask_sz: u8,
    pub blue_pos: u8,
    pub blue_mask_sz: u8,
}

pub struct SectionInfo {
    pub num_shdrs: usize,
    pub shdrs: *const Elf64Shdr,
    pub shstrtab_idx: usize,
}

pub fn get_info() -> BootloaderInfo {
    multiboot::Multiboot::get_info()
}
