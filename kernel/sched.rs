// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::cell::OnceCell;

use crate::arch;
use crate::bootloader::BootloaderInfo;
use crate::process::{Process, State};
use crate::small_vec::SmallVec;
use crate::spinlock::Mutex;

static LOOP_ELF: &[u8] = include_bytes!("../build/loop");

static SCHEDULER: Mutex<OnceCell<Scheduler>> = Mutex::new(OnceCell::new());

struct Scheduler {
    current_idx: usize,
    processes: SmallVec<Process>,
}

impl Scheduler {
    fn new() -> Self {
        Self {
            current_idx: 0,
            processes: SmallVec::new(),
        }
    }

    fn get_next(&self) -> TaskSwitch {
        for proc in self.processes.iter_round_robin(self.current_idx) {
            if proc.state == State::Runnable {
                return TaskSwitch::NewTask(*proc);
            }
        }

        let current = self.processes[self.current_idx];

        if current.state == State::Running {
            return TaskSwitch::SameTask(current);
        }

        TaskSwitch::Idle
    }

    fn preempt_current(&mut self) {
        self.processes[self.current_idx].state = State::Runnable;
    }
}

enum TaskSwitch {
    NewTask(Process),
    SameTask(Process),
    Idle,
}

pub fn init(info: &BootloaderInfo) {
    let mut sched = Scheduler::new();

    sched.processes.push(Process::from_elf(LOOP_ELF, info));
    sched.processes[0].state = State::Running;

    assert!(SCHEDULER.lock().set(sched).is_ok());
}

pub fn next() {
    let mut cell = SCHEDULER.lock();
    let sched = cell.get_mut().unwrap();

    match sched.get_next() {
        TaskSwitch::NewTask(proc) => {
            sched.preempt_current();
            run(proc)
        }
        TaskSwitch::SameTask(proc) => run(proc),
        TaskSwitch::Idle => idle(),
    }
}

fn run(_proc: Process) {
}

fn idle() {
    arch::interrupts::enable();

    loop {
        arch::asm::idle();
    }
}
