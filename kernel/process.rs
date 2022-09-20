// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bootloader::BootloaderInfo;
use crate::mm::types::{RegisterFrameOps, RootPageDirOps};
use crate::{arch, elf};

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
            root_dir: arch::RootPageDir::new_userspace(info),
            registers: arch::RegisterFrame::new_userspace(),
            state: State::Runnable,
        };

        elf::load(&mut process, bytes);

        process
    }
}
