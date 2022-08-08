// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::bootloader::BootloaderInfo;
use crate::process::{Process, State};

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

    fn get_next(&self) -> Option<&Process> {
        for idx in RoundRobinIterator::new(self.current_idx, MAX_PROCESSES) {
            if let Some(proc) = self.processes[idx].as_ref() && proc.state == State::Runnable {
                return Some(proc);
            }
        }

        if let Some(current) = self.current() && current.state == State::Running {
            return Some(current);
        }

        None
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

pub fn init(info: &BootloaderInfo) {
    unsafe {
        SCHEDULER.processes[0] = Some(Process::from_elf(LOOP_ELF, info));
    }
}

pub fn next() {
    unsafe {
        match SCHEDULER.get_next() {
            Some(proc) => run(proc),
            None => idle(),
        }
    }
}

fn run(_proc: &Process) {
}

fn idle() {
}
