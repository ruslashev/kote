// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

mod multiboot;

use core::fmt;
use core::{slice, str};

use crate::elf::Elf64Shdr;
use crate::units::PowerOfTwoOps;

const MMAP_MAX_ENTRIES: usize = 32;

trait Bootloader {
    fn get_info() -> BootloaderInfo;
}

pub struct BootloaderInfo {
    pub free_areas: MemoryMap,
    pub framebuffer: FramebufferInfo,
    pub section_headers: Option<SectionInfo>,
}

pub struct MemoryMap {
    pub entries: [Region; MMAP_MAX_ENTRIES],
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

#[derive(Debug)]
pub struct SectionInfo {
    pub num_shdrs: usize,
    pub shdrs: *const Elf64Shdr,
    pub shstrtab_idx: usize,
}

#[derive(Debug, Clone)]
pub struct SectionInfoIterator<'a> {
    idx: usize,
    shstrtab: &'a [u8],
    info: &'a SectionInfo,
}

impl<'i> SectionInfoIterator<'i> {
    pub fn from_info(info: &'i SectionInfo) -> Self {
        let shstrtab = unsafe {
            let idx = info.shstrtab_idx;
            let shdr = info.shdrs.add(idx).read();
            let addr = shdr.sh_addr as *const u8;
            let size = shdr.sh_size as usize;
            slice::from_raw_parts(addr, size)
        };

        SectionInfoIterator {
            idx: 0,
            shstrtab,
            info,
        }
    }
}

impl<'a> Iterator for SectionInfoIterator<'a> {
    type Item = (&'a str, &'a Elf64Shdr);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.info.num_shdrs {
            let shdr = unsafe { &*self.info.shdrs.add(self.idx) };

            let name_beg = shdr.sh_name as usize;
            let mut name_end = None;

            for (end_idx, &ch) in self.shstrtab.iter().enumerate().skip(name_beg) {
                if ch == 0 {
                    name_end = Some(end_idx);
                    break;
                }
            }

            assert!(name_end.is_some(), "Kernel shstrtab overflow");

            let name_end = name_end.unwrap();
            let name = if name_end == 0 {
                "null"
            } else {
                str::from_utf8(&self.shstrtab[name_beg..name_end]).unwrap()
            };

            self.idx += 1;
            Some((name, shdr))
        } else {
            None
        }
    }
}

impl fmt::Display for SectionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, shdr) in SectionInfoIterator::from_info(self) {
            let sh_type = shdr.sh_type;
            let sh_flags = shdr.sh_flags;
            let sh_addr = shdr.sh_addr;
            let sh_size = shdr.sh_size;

            writeln!(f, "{:15} {} {:06b} {:16x} {}", name, sh_type, sh_flags, sh_addr, sh_size)?;
        }

        Ok(())
    }
}

impl fmt::Display for MemoryMap {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut npages = 0;

        for idx in 0..self.num_entries {
            let Region { start, end } = self.entries[idx];

            let pg_start = start.page_round_down();
            let pg_end = end.page_round_up();
            npages += (pg_end - pg_start) / 4096;

            writeln!(f, "{:x}..{:<16x}", start, end)?;
        }

        writeln!(f, "npages={}", npages)?;

        Ok(())
    }
}

pub fn get_info() -> BootloaderInfo {
    multiboot::Multiboot::get_info()
}
