// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::exceptions::ExceptionFrame;
use crate::sched;
use crate::types::PowerOfTwoOps;

extern "C" {
    fn stack_guard_top();
    fn stack_guard_bot();
}

pub(super) fn divide_by_zero(_frame: &ExceptionFrame) {
    println!("Divide by zero handler");
}

pub(super) fn breakpoint(_frame: &ExceptionFrame) {
    // Usually this would be the place to enter a debugger, but the breakpoint exception is only
    // used to test interrupts in userspace.
    sched::next();
}

pub(super) fn page_fault(_frame: &ExceptionFrame) {
    let vaddr = read_reg!(cr2);
    let round = vaddr.page_round_down() as usize;
    let guard_top_low = (stack_guard_top as usize) & 0xffffffff;
    let guard_bot_low = (stack_guard_bot as usize) & 0xffffffff;

    if round == guard_top_low {
        panic!("Kernel stack overflow");
    }

    if round == guard_bot_low {
        panic!("Kernel stack underflow");
    }

    println!("Faulting addr = {:#x}", vaddr);
}
