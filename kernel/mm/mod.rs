// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod pg_alloc;
pub mod types;

use crate::arch;
use crate::bootloader::BootloaderInfo;

pub fn init(info: &mut BootloaderInfo) -> usize {
    arch::mmu::init();

    let page_infos_end = pg_alloc::init(info);

    usize::try_from(page_infos_end).expect("framebuffer address overflows usize")
}
