// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bootloader::BootloaderInfo;
use crate::mm::types::{RegisterFrameOps, RootPageDirOps};
use crate::{arch, elf, mm};

static LOOP_ELF: &[u8] = include_bytes!("../build/loop");

// This doesn't have to a be a static array, but it is convenient for now
pub const MAX_PROCESSES: usize = 32;
pub static mut PROCESSES: [Option<Process>; MAX_PROCESSES] = [EMPTY_PROCESS; MAX_PROCESSES];

const EMPTY_PROCESS: Option<Process> = None;

pub struct Process {
    pub root_dir: arch::RootPageDir,
    pub registers: arch::RegisterFrame,
    pub state: State,
}

#[derive(PartialEq, Eq)]
pub enum State {
    Runnable,
    Running,
    Stopped,
}

impl Process {
    fn from_elf(bytes: &[u8], info: &BootloaderInfo) -> Self {
        let mut process = Process {
            root_dir: arch::RootPageDir::new_userspace_root_dir(info),
            registers: arch::RegisterFrame::default(),
            state: State::Runnable,
        };

        process.root_dir.switch_to_this();

        process.registers.set_stack_pointer(arch::USER_STACK_START.0 + arch::USER_STACK_SIZE);

        elf::load(&mut process, bytes);

        mm::switch_to_kernel_root_dir();

        process
    }

    pub fn run(&self) {
    }
}

pub fn init(info: &BootloaderInfo) {
    unsafe {
        PROCESSES[0] = Some(Process::from_elf(LOOP_ELF, info));
    }
}
