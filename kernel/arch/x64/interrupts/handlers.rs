// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::exceptions::ExceptionFrame;

pub(super) fn divide_by_zero(_frame: &ExceptionFrame) {
    println!("Divide by zero handler");
}

pub(super) fn page_fault(_frame: &ExceptionFrame) {
    let vaddr = read_reg!(cr2);
    let _offset = (vaddr & 0x000000000fff) >> 0;
    let _pt_off = (vaddr & 0x0000001ff000) >> 12;
    let _pd_off = (vaddr & 0x00003fe00000) >> 21;
    let _pdpt_o = (vaddr & 0x007fc0000000) >> 30;
    let _pml4_o = (vaddr & 0xff8000000000) >> 39;

    println!("Faulting addr = {:#x}", vaddr);
}
