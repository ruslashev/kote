// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Routines for parsing multiboot info.
// Contains excerpts from https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html

use core::mem::size_of;

use crate::panic::panic_early;
use crate::utils;

extern "C" {
    static mb_info: u64;
}

pub fn init() {
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

    if !utils::is_po2_aligned(start, alignment) {
        panic_early("Multiboot information structure is not aligned");
    }

    let mut total_size = unsafe { (start as *const u32).read() } as u64;

    start += (size_of::<u32>() as u64) * 2;

    while total_size > 0 {
        let header = start as *const u32;
        let tag_type = unsafe { header.read() };
        let tag_size = unsafe { header.offset(1).read() };
        let aligned_size = utils::po2_round_up(tag_size as u64, alignment);

        match tag_type {
            0 => break,
            4 => parse_mem_info(header),
            6 => parse_mem_map(header),
            _ => {}
        }

        start += aligned_size;
        total_size -= aligned_size;
    }
}

fn parse_mem_info(header: *const u32) {
    /*     +-------------------+
     * u32 | type = 4          |
     * u32 | size = 16         |
     * u32 | mem_lower         |
     * u32 | mem_upper         |
     *     +-------------------+
     *
     * `mem_lower` and `mem_upper` indicate the amount of lower and upper memory, respectively, in
     * kilobytes. Lower memory starts at address 0, and upper memory starts at address 1 megabyte.
     * The maximum possible value for lower memory is 640 kilobytes. The value returned for upper
     * memory is maximally the address of the first upper memory hole minus 1 megabyte. It is not
     * guaranteed to be this value.
     */

    let _mem_lower = unsafe { header.offset(2).read() };
    let _mem_upper = unsafe { header.offset(3).read() };
}

fn parse_mem_map(header: *const u32) {
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
        length:    u64,
        etype:     u32,
        reserved:  u32,
    }

    let tag_size = unsafe { header.offset(1).read() };
    let entry_size = unsafe { header.offset(2).read() };
    let entry_version = unsafe { header.offset(3).read() };

    if entry_version != 0 {
        panic_early("Multiboot memory map version has unexpected non-zero value");
    }

    let mut entries = unsafe { header.offset(4).cast::<Entry>() };
    let mut total_size = 0;

    while total_size < tag_size {
        let entry = unsafe { entries.read() };
        let base_addr = entry.base_addr;
        let length = entry.length;
        let etype = entry.etype;

        printk!("addr={:<12x} len={:<12} type={}", base_addr, length, etype);

        unsafe {
            entries = entries.add(1);
        }

        total_size += entry_size;
    }
}
