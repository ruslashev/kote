// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod pg_alloc;
pub mod types;

use crate::arch;
use crate::bootloader::BootloaderInfo;

pub fn init(info: &mut BootloaderInfo) -> types::VirtAddr {
    arch::mmu::init();

    pg_alloc::init(info)
}
