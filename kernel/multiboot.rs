// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

// Routines for parsing multiboot info.

use core::mem::size_of;

use crate::panic::panic_early;
use crate::utils;

extern "C" {
    static mb_info: u64;
}

pub fn init() {
    /* Excerpt from https://www.gnu.org/software/grub/manual/multiboot2/multiboot.html:
     *
     * Boot information consists of fixed part and a series of tags.
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
            _ => {}
        }

        start += aligned_size;
        total_size -= aligned_size;
    }
}
