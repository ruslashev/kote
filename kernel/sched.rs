// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::ops::{Deref, DerefMut};

use crate::arch;
use crate::bootloader::BootloaderInfo;
use crate::process::{Process, State};
use crate::small_vec::SmallVec;
use crate::spinlock::{Mutex, SpinlockGuard};

static SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::empty());

struct Scheduler {
    processes: SmallVec<Process>,
}

pub struct ProcessGuard<'s> {
    sched: SpinlockGuard<'s, Scheduler>,
}

impl Scheduler {
    const fn empty() -> Self {
        Self {
            processes: SmallVec::empty(),
        }
    }

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
    static READ_ELF: &[u8] = include_bytes!("../build/bundle/input");

    let mut sched = Scheduler::new();

    sched.processes.push_back(Process::from_elf("loop", LOOP_ELF, info));
    sched.processes.push_back(Process::from_elf("breakpoint", BKPT_ELF, info));
    sched.processes.push_back(Process::from_elf("loop 2", LOOP_ELF, info));
    sched.processes.push_back(Process::from_elf("hello_world", HLWD_ELF, info));
    sched.processes.push_back(Process::from_elf("input", READ_ELF, info));

    *SCHEDULER.lock() = sched;
}

pub fn next() -> ! {
    let mut sched = SCHEDULER.lock();

    match sched.get_next() {
        TaskSwitch::NewTask(new_idx, proc) => {
            trace!("switching to a new task '{}'", proc.name);
            sched.set_current(new_idx);
            drop(sched);
            run(proc);
        }
        TaskSwitch::SameTask(proc) => {
            trace!("switching to the same task '{}'", proc.name);
            drop(sched);
            run(proc);
        }
        TaskSwitch::Idle => {
            trace!("idle");
            drop(sched);
            idle();
        }
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

impl Deref for ProcessGuard<'_> {
    type Target = Process;

    fn deref(&self) -> &Process {
        self.sched.processes.current().unwrap()
    }
}

impl DerefMut for ProcessGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.sched.processes.current().unwrap()
    }
}

pub fn current<'s>() -> ProcessGuard<'s> {
    ProcessGuard {
        sched: SCHEDULER.lock(),
    }
}
