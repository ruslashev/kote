// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::process::{PROCESSES, MAX_PROCESSES, State};

static mut SCHEDULER: Scheduler = Scheduler { proc_idx: 0 };

struct Scheduler {
    proc_idx: usize,
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

pub unsafe fn next() {
    for idx in RoundRobinIterator::new(SCHEDULER.proc_idx, MAX_PROCESSES) {
        if let Some(proc) = &PROCESSES[idx] && proc.state == State::Runnable {
            proc.run();
        }
    }

    let current = PROCESSES[SCHEDULER.proc_idx].as_ref().unwrap();

    if current.state == State::Running {
        current.run();
    }
}
