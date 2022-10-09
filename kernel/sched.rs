// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::cell::OnceCell;

use crate::arch;
use crate::bootloader::BootloaderInfo;
use crate::process::{Process, State};
use crate::small_vec::SmallVec;
use crate::spinlock::Mutex;

static SCHEDULER: Mutex<OnceCell<Scheduler>> = Mutex::new(OnceCell::new());

struct Scheduler {
    processes: SmallVec<Process>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            processes: SmallVec::new(),
        }
    }

    fn get_next(&mut self) -> TaskSwitch {
        for (idx, proc) in self.processes.iter_round_robin() {
            if proc.state == State::Runnable {
                return TaskSwitch::NewTask(idx, *proc);
            }
        }

        if let Some(current) = self.processes.current() && current.state == State::Running {
            return TaskSwitch::SameTask(*current);
        }

        TaskSwitch::Idle
    }

    fn set_current(&mut self, new_idx: usize) {
        if let Some(current) = self.processes.current() {
            current.state = State::Runnable;
        }

        self.processes.set_current(new_idx);

        self.processes.current().unwrap().state = State::Running;
    }
}

enum TaskSwitch {
    NewTask(usize, Process),
    SameTask(Process),
    Idle,
}

pub fn init(info: &BootloaderInfo) {
    static LOOP_ELF: &[u8] = include_bytes!("../build/bundle/loop");
    static BKPT_ELF: &[u8] = include_bytes!("../build/bundle/breakpoint");
    static HLWD_ELF: &[u8] = include_bytes!("../build/bundle/hello_world");

    let mut sched = Scheduler::new();

    sched.processes.push_back(Process::from_elf("loop", LOOP_ELF, info));
    sched.processes.push_back(Process::from_elf("breakpoint", BKPT_ELF, info));
    sched.processes.push_back(Process::from_elf("loop 2", LOOP_ELF, info));
    sched.processes.push_back(Process::from_elf("hello_world", HLWD_ELF, info));

    assert!(SCHEDULER.lock().set(sched).is_ok());
}

pub fn next() -> ! {
    let mut cell = SCHEDULER.lock();
    let sched = cell.get_mut().unwrap();

    match sched.get_next() {
        TaskSwitch::NewTask(new_idx, proc) => {
            trace!("switching to new task '{}'", proc.name);
            sched.set_current(new_idx);
            drop(cell);
            run(proc);
        }
        TaskSwitch::SameTask(proc) => {
            trace!("switching to same task '{}'", proc.name);
            run(proc);
        }
        TaskSwitch::Idle => idle(),
    }
}

fn run(proc: Process) -> ! {
    arch::switch_to_process(proc);
}

fn idle() -> ! {
    arch::interrupts::enable();

    loop {
        arch::asm::idle();
    }
}

// TODO: this should probably return `&mut Process` instead of `Option<Process>`
pub fn current() -> Option<Process> {
    let cell = SCHEDULER.lock();
    let sched = cell.get().unwrap();

    sched.processes.current().copied()
}
