// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::cell::OnceCell;

use crate::arch;
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
    }
}

enum TaskSwitch {
    NewTask(usize, Process),
    SameTask(Process),
    Idle,
}

pub fn init() {
    static LOOP_ELF: &[u8] = include_bytes!("../build/loop");

    let mut sched = Scheduler::new();

    sched.processes.push_back(Process::from_elf(LOOP_ELF));

    assert!(SCHEDULER.lock().set(sched).is_ok());
}

pub fn next() {
    let mut cell = SCHEDULER.lock();
    let sched = cell.get_mut().unwrap();

    match sched.get_next() {
        TaskSwitch::NewTask(new_idx, proc) => {
            sched.set_current(new_idx);
            run(proc);
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
