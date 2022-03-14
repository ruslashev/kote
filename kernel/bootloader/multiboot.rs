// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Routines for parsing multiboot info.
// Contains excerpts from https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html

use core::mem::size_of;
use core::ops::Range;

use super::*;
use crate::elf::Elf64Shdr;
use crate::panic::panic_no_graphics;
use crate::units;

extern "C" {
    static mb_info: u64;
}

pub struct Multiboot;

impl Bootloader for Multiboot {
    fn get_info() -> BootloaderInfo {
        parse()
    }
}

/// Parse multiboot information structure located address `mb_info`, stored during boot in start.s.
fn parse() -> BootloaderInfo {
    /* Boot information consists of fixed part and a series of tags.
     * Its start is 8-bytes aligned. Fixed part is as following:
     *
     *     +-------------------+
     * u32 | total_size        |
     * u32 | reserved          |
     *     +-------------------+
     *
     * `total_size` contains the total size of boot information including this field and terminating
     * tag in bytes.
     *
     * `reserved` is always set to zero and must be ignored by OS image.
     *
     * Every tag begins with following fields:
     *
     *     +-------------------+
     * u32 | type              |
     * u32 | size              |
     *     +-------------------+
     *
     * `type` contains an identifier of contents of the rest of the tag. `size` contains the size
     * of tag including header fields but not including padding. Tags follow one another padded when
     * necessary in order for each tag to start at 8-bytes aligned address. Tags are terminated by a
     * tag of type `0` and size `8`.
     */

    let mut start = unsafe { mb_info };
    let alignment = 8;

    if !units::is_po2_aligned(start, alignment) {
        panic_no_graphics("Multiboot information structure is not aligned");
    }

    let mut total_size = unsafe { (start as *const u32).read() } as u64;

    start += (size_of::<u32>() as u64) * 2;

    let mut mmap = None;
    let mut fb = None;
    let mut shdrs = None;

    while total_size > 0 {
        let header = start as *const u32;
        let tag_type = unsafe { header.read() };
        let tag_size = unsafe { header.offset(1).read() };
        let aligned_size = units::po2_round_up(tag_size as u64, alignment);

        match tag_type {
            0 => break,
            6 => mmap = Some(parse_mem_map(header)),
            8 => fb = Some(parse_framebuffer_info(header)),
            9 => shdrs = Some(parse_elf_sections(header)),
            _ => {}
        }

        start += aligned_size;
        total_size -= aligned_size;
    }

    let mut info = BootloaderInfo {
        memory_map: mmap.unwrap_or_else(|| panic_no_graphics("Multiboot: mmap tag not found")),
        framebuffer: fb
            .unwrap_or_else(|| panic_no_graphics("Multiboot: framebuffer tag not found")),
        section_headers: shdrs,
    };

    remove_reserved_areas(&mut info);

    info
}

fn parse_mem_map(header: *const u32) -> MemoryMap {
    /*        +-------------------+
     * u32    | type = 6          |
     * u32    | size              |
     * u32    | entry_size        |
     * u32    | entry_version     |
     * varies | entries           |
     *        +-------------------+
     *
     * `entry_size` contains the size of one entry so that in future new fields may be added to it.
     * It's guaranteed to be a multiple of 8. `entry_version` is currently set at `0`. Future
     * versions will increment this field. Future version are guranteed to be backward compatible
     * with older format. Each entry has the following structure:
     *
     *        +-------------------+
     * u64    | base_addr         |
     * u64    | length            |
     * u32    | type              |
     * u32    | reserved          |
     *        +-------------------+
     *
     * `size` contains the size of current entry including this field itself. It may be bigger than
     * 24 bytes in future versions but is guaranteed to be `base_addr` is the starting physical
     * address. `length` is the size of the memory region in bytes. `type` is the variety of address
     * range represented, where a value of 1 indicates available RAM, value of 3 indicates usable
     * memory holding ACPI information, value of 4 indicates reserved memory which needs to be
     * preserved on hibernation, value of 5 indicates a memory which is occupied by defective RAM
     * modules and all other values currently indicated a reserved area. `reserved` is set to `0` by
     * bootloader and must be ignored by the OS image.
     */

    #[repr(C, packed)]
    struct Entry {
        base_addr: u64,
        length: u64,
        etype: u32,
        reserved: u32,
    }

    let tag_size = unsafe { header.offset(1).read() };
    let entry_size = unsafe { header.offset(2).read() };
    let entry_version = unsafe { header.offset(3).read() };

    if entry_version != 0 {
        panic_no_graphics("Multiboot memory map version has unexpected non-zero value");
    }

    let mut entries = unsafe { header.offset(4).cast::<Entry>() };
    let mut total_size = 0;
    let empty = Region { start: 0, end: 0 };
    let mut mmap = [empty; MMAP_MAX_ENTRIES];
    let mut mmap_entry = 0;

    while total_size < tag_size {
        let entry = unsafe { entries.read() };
        let start = entry.base_addr as usize;
        let length = entry.length as usize;
        let end = start + length;

        if mmap_entry >= MMAP_MAX_ENTRIES {
            panic_no_graphics("Multiboot: mmap entry overflow");
        }

        // Available
        if entry.etype == 1 {
            mmap[mmap_entry] = Region { start, end };
            mmap_entry += 1;
        }

        unsafe {
            entries = entries.add(1);
        }

        total_size += entry_size;
    }

    MemoryMap {
        entries: mmap,
        num_entries: mmap_entry,
    }
}

fn parse_framebuffer_info(header: *const u32) -> FramebufferInfo {
    /*        +--------------------+
     * u32    | type = 8           |
     * u32    | size               |
     * u64    | framebuffer_addr   |
     * u32    | framebuffer_pitch  |
     * u32    | framebuffer_width  |
     * u32    | framebuffer_height |
     * u8     | framebuffer_bpp    |
     * u8     | framebuffer_type   |
     * u8     | reserved           |
     * varies | color_info         |
     *        +--------------------+
     *
     * The field `framebuffer_addr` contains framebuffer physical address. This field is 64-bit wide
     * but bootloader should set it under 4GiB if possible for compatibility with payloads which
     * aren't aware of PAE or amd64. The field `framebuffer_pitch` contains pitch in bytes. The
     * fields `framebuffer_width`, `framebuffer_height` contain framebuffer dimensions in pixels.
     * The field `framebuffer_bpp` contains number of bits per pixel. `reserved` always contains 0
     * in current version of specification and must be ignored by OS image.
     *
     * If `framebuffer_type` is set to `1` it means direct RGB color. Then color_type is defined as
     * follows:
     *
     *       +----------------------------------+
     * u8    | framebuffer_red_field_position   |
     * u8    | framebuffer_red_mask_size        |
     * u8    | framebuffer_green_field_position |
     * u8    | framebuffer_green_mask_size      |
     * u8    | framebuffer_blue_field_position  |
     * u8    | framebuffer_blue_mask_size       |
     *       +----------------------------------+
     */

    #[repr(C, packed)]
    #[derive(Debug, Clone, Copy)]
    struct FrameBufferTag {
        ttype: u32,
        size: u32,
        addr: u64,
        pitch: u32,
        width: u32,
        height: u32,
        bpp: u8,
        ftype: u8,
        reserved: u16,
        red_pos: u8,
        red_mask_sz: u8,
        green_pos: u8,
        green_mask_sz: u8,
        blue_pos: u8,
        blue_mask_sz: u8,
    }

    let fb = unsafe { header.cast::<FrameBufferTag>().read() };

    FramebufferInfo {
        addr: fb.addr,
        width: fb.width,
        height: fb.height,
        pitch: fb.pitch,
        bpp: fb.bpp,
        red_pos: fb.red_pos,
        red_mask_sz: fb.red_mask_sz,
        green_pos: fb.green_pos,
        green_mask_sz: fb.green_mask_sz,
        blue_pos: fb.blue_pos,
        blue_mask_sz: fb.blue_mask_sz,
    }
}

fn parse_elf_sections(header: *const u32) -> SectionInfo {
    // The diagram in the documentation states that fields num, entsize and shndx are u16, but it is
    // clearly outdated, as even the code in that same page declares that fields are, in fact, u32.

    /*        +-------------------+
     * u32    | type = 9          |
     * u32    | size              |
     * u32    | num               |
     * u32    | entsize           |
     * u32    | shndx             |
     * varies | section headers   |
     *        +-------------------+
     */

    unsafe {
        let num_shdrs = header.offset(2).read() as usize;
        let shstrtab_idx = header.offset(4).read() as usize;
        let shdrs = header.offset(5).cast::<Elf64Shdr>();

        SectionInfo {
            num_shdrs,
            shdrs,
            shstrtab_idx,
        }
    }
}

fn remove_reserved_areas(info: &mut BootloaderInfo) {
    if info.section_headers.is_none() {
        panic!("Systems using Multiboot require kernel section headers tag to be present");
    }

    let mmap = &mut info.memory_map;
    let shdrs = &info.section_headers.as_ref().unwrap();
    let fb = &info.framebuffer;
    let fb_addr = fb.addr as usize;

    let first_page = 0..0x1000;
    let io_hole = 0xa0000..0x100000;
    let fb_range = fb_addr..fb_addr + (fb.height * fb.pitch) as usize;

    let some_reserved = [first_page, io_hole, fb_range];
    let shdr_ranges = SectionInfoIterator::from_info(shdrs).map(|(_, shdr)| {
        let a = shdr.sh_addr as usize;
        let s = shdr.sh_size as usize;
        a..a + s
    });

    let mut keep_looping;

    loop {
        keep_looping = false;

        'restart: for eidx in 0..mmap.num_entries {
            let entry = mmap.entries[eidx];
            let all_reserved = shdr_ranges.clone().chain(some_reserved.clone());

            for reserved in all_reserved {
                let added = resolve_overlaps(
                    eidx,
                    &entry,
                    &reserved,
                    &mut mmap.entries,
                    &mut mmap.num_entries,
                );

                if added {
                    keep_looping = true;
                    break 'restart;
                }
            }
        }

        if !keep_looping {
            break;
        }
    }

    cleanup_empty_ranges(&mut mmap.entries, &mut mmap.num_entries);

    sort_ranges(&mut mmap.entries, mmap.num_entries);
}

fn resolve_overlaps(
    eidx: usize,
    entry: &Region,
    reserved: &Range<usize>,
    entries: &mut [Region; MMAP_MAX_ENTRIES],
    num_entries: &mut usize,
) -> bool {
    let r = reserved;
    let e = entry;

    // Ignore empty ranges
    if (r.start == 0 && r.end == 0) || (e.start == 0 && e.end == 0) {
        return false;
    }

    // No overlap
    // └──────────┘               e
    //               └──────────┘ r
    //               └──────────┘ e
    // └──────────┘               r
    if r.end <= e.start || e.end <= r.start {
        return false;
    }

    // Overlap and `reserved` is to the left
    //         └──────────┘ e
    // └──────────┘         r
    if e.start < r.end && e.start >= r.start {
        entries[eidx].start = r.end;
        return false;
    }

    // Overlap and `reserved` is to the right
    // └──────────┘         e
    //         └──────────┘ r
    if e.end > r.start && e.end <= r.end {
        entries[eidx].end = r.start;
        return false;
    }

    // `entry` is completely inside `reserved`
    //       └──────┘    e
    //    └────────────┘ r
    if r.start <= e.start && r.end >= e.end {
        entries[eidx].start = 0;
        entries[eidx].end = 0;
        return false;
    }

    // `reserved` is completely inside `entry`
    //    └────────────┘ e
    //       └──────┘    r
    if e.start <= r.start && e.end >= r.end {
        entries[eidx].end = r.start;

        if *num_entries >= MMAP_MAX_ENTRIES {
            panic_no_graphics("Multiboot: mmap entry overflow while resolving overlaps");
        }

        entries[*num_entries].start = r.end;
        entries[*num_entries].end = e.end;
        *num_entries += 1;
        return true;
    }

    panic!("unexpected range configuration");
}

fn cleanup_empty_ranges(entries: &mut [Region; MMAP_MAX_ENTRIES], num_entries: &mut usize) {
    let old_num_entries = *num_entries;

    for eidx in 0..old_num_entries {
        if entries[eidx].start == 0 && entries[eidx].end == 0 {
            let ptr = entries.as_mut_ptr();

            unsafe {
                let src = ptr.add(eidx + 1);
                let dst = ptr.add(eidx);

                core::ptr::copy(src, dst, *num_entries - eidx - 1);
            }

            *num_entries -= 1;
        }
    }
}

fn sort_ranges(entries: &mut [Region; MMAP_MAX_ENTRIES], num_entries: usize) {
    for i in 1..num_entries {
        let key = entries[i];
        let mut j = i as isize - 1;

        while j >= 0 && entries[j as usize].start > key.start {
            entries[j as usize + 1] = entries[j as usize];
            j -= 1;
        }

        entries[(j + 1) as usize] = key;
    }
}
