// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bootloader::BootloaderInfo;
use crate::mm::types::{RegisterFrameOps, RootPageDirOps};
use crate::{arch, elf, mm};

#[derive(Copy, Clone)]
pub struct Process {
    pub root_dir: arch::RootPageDir,
    pub registers: arch::RegisterFrame,
    pub state: State,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum State {
    Runnable,
    Running,
    Stopped,
}

impl Process {
    pub fn from_elf(bytes: &[u8], info: &BootloaderInfo) -> Self {
        let mut process = Process {
            root_dir: arch::RootPageDir::new_userspace_root_dir(info),
            registers: arch::RegisterFrame::default(),
            state: State::Runnable,
        };

        process.root_dir.switch_to_this();

        process.registers.set_stack_pointer(arch::USER_STACK_START.0 + arch::USER_STACK_SIZE);
        process.registers.enable_interrupts();

        elf::load(&mut process, bytes);

        mm::switch_to_kernel_root_dir();

        process
    }
}
