// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::mm::types::{RegisterFrameOps, RootPageDirOps};
use crate::spinlock::SpinlockMutex;
use crate::{arch, elf};

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
    fn from_elf(bytes: &[u8]) -> Self {
        let root_dir = arch::RootPageDir::new();
        let mut process = Process {
            root_dir,
            registers: arch::RegisterFrame::default(),
        };

        process.root_dir.switch_to_this();

        process.root_dir.alloc_range(
            arch::USER_STACK_START,
            arch::USER_STACK_SIZE,
            arch::mmu::WRITABLE | arch::mmu::USER_ACCESSIBLE,
        );
        process
            .registers
            .set_stack_pointer(arch::USER_STACK_START.0 + arch::USER_STACK_SIZE);

        elf::load(&mut process, bytes);

        process
    }
}

pub fn init() {
    println!("{:x?}", &LOOP_ELF[0..4]);
}
