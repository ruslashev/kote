// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bootloader::BootloaderInfo;
use crate::mm::types::{RegisterFrameOps, RootPageDirOps};
use crate::spinlock::SpinlockMutex;
use crate::{arch, elf, mm};

static LOOP_ELF: &[u8] = include_bytes!("../build/loop");

// This doesn't have to a be a static array, but it is convenient for now
const MAX_PROCESSES: usize = 32;
const EMPTY_PROCESS: Option<Process> = None;
static PROCESSES: SpinlockMutex<[Option<Process>; MAX_PROCESSES]> =
    SpinlockMutex::new([EMPTY_PROCESS; MAX_PROCESSES]);

pub struct Process {
    pub root_dir: arch::RootPageDir,
    pub registers: arch::RegisterFrame,
}

impl Process {
    fn from_elf(bytes: &[u8], info: &BootloaderInfo) -> Self {
        let mut process = Process {
            root_dir: arch::RootPageDir::new_userspace_root_dir(info),
            registers: arch::RegisterFrame::default(),
        };

        process.root_dir.switch_to_this();

        process.registers.set_stack_pointer(arch::USER_STACK_START.0 + arch::USER_STACK_SIZE);

        elf::load(&mut process, bytes);

        mm::switch_to_kernel_root_dir();

        process
    }
}

pub fn init(info: &BootloaderInfo) {
    let mut processes = PROCESSES.lock();

    processes[0] = Some(Process::from_elf(LOOP_ELF, info));
}
