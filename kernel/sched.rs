// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bootloader::BootloaderInfo;
use crate::process::{Process, State};
use crate::arch;

static LOOP_ELF: &[u8] = include_bytes!("../build/loop");

const MAX_PROCESSES: usize = 32;

static mut SCHEDULER: Scheduler = Scheduler::new();

struct Scheduler {
    current_idx: usize,
    processes: [Option<Process>; MAX_PROCESSES],
}

impl Scheduler {
    const EMPTY_PROCESS: Option<Process> = None;

    const fn new() -> Self {
        Self {
            current_idx: 0,
            processes: [Self::EMPTY_PROCESS; MAX_PROCESSES],
        }
    }

    fn current(&self) -> Option<&Process> {
        self.processes[self.current_idx].as_ref()
    }

    fn current_mut(&mut self) -> Option<&mut Process> {
        self.processes[self.current_idx].as_mut()
    }

    fn get_next(&self) -> TaskSwitch {
        for idx in RoundRobinIterator::new(self.current_idx, MAX_PROCESSES) {
            if let Some(proc) = self.processes[idx].as_ref() && proc.state == State::Runnable {
                return TaskSwitch::NewTask(proc);
            }
        }

        if let Some(current) = self.current() && current.state == State::Running {
            return TaskSwitch::SameTask(current);
        }

        TaskSwitch::Idle
    }

    fn preempt_current(&mut self) {
        if let Some(current) = self.current_mut() {
            current.state = State::Runnable;
        }
    }
}

struct RoundRobinIterator {
    index: usize,
    start: usize,
    range: usize,
}

impl Iterator for RoundRobinIterator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.index += 1;
        self.index %= self.range;

        if self.index == self.start {
            None
        } else {
            Some(self.index)
        }
    }
}

impl RoundRobinIterator {
    fn new(start: usize, range: usize) -> Self {
        Self {
            index: start,
            start,
            range,
        }
    }
}

enum TaskSwitch<'a> {
    NewTask(&'a Process),
    SameTask(&'a Process),
    Idle,
}

pub fn init(info: &BootloaderInfo) {
    unsafe {
        let mut loop_proc = Process::from_elf(LOOP_ELF, info);
        loop_proc.state = State::Running;

        SCHEDULER.processes[0] = Some(loop_proc);
    }
}

pub fn next() {
    unsafe {
        match SCHEDULER.get_next() {
            TaskSwitch::NewTask(proc) => {
                SCHEDULER.preempt_current();
                run(proc);
            }
            TaskSwitch::SameTask(proc) => run(proc),
            TaskSwitch::Idle => idle(),
        }
    }
}

fn run(_proc: &Process) {
}

fn idle() {
    arch::interrupts::enable();

    loop {
        arch::asm::idle();
    }
}
