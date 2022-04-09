// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::exceptions::ExceptionFrame;

pub(super) fn divide_by_zero(_frame: &ExceptionFrame) {
    println!("Divide by zero handler");
}

pub(super) fn page_fault(_frame: &ExceptionFrame) {
    let vaddr = read_reg!(cr2);

    println!("Faulting addr = {:#x}", vaddr);
}
