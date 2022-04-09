// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub mod addr;
mod pg_alloc;

use crate::arch;
use crate::bootloader::BootloaderInfo;

pub fn init(info: &BootloaderInfo) {
    arch::mmu::init();
    pg_alloc::init(info);
}
