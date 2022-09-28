// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::arch::asm;

const PORT_CMND: u16 = 0x70;
const PORT_DATA: u16 = 0x71;

const REG_A: u8 = 0xa;
const REG_B: u8 = 0xb;
const REG_C: u8 = 0xc;

const INTERRUPT_ENABLE: u8 = 1 << 6;

pub(super) fn init() {
    asm::nmi_disable();

    let prev = read_register(REG_B);
    write_register(REG_B, prev | INTERRUPT_ENABLE);

    set_frequency();

    asm::nmi_enable();
}

fn read_register(reg: u8) -> u8 {
    asm::io::outb(PORT_CMND, reg);
    asm::io::inb(PORT_DATA)
}

fn write_register(reg: u8, val: u8) {
    asm::io::outb(PORT_CMND, reg);
    asm::io::outb(PORT_DATA, val);
}

fn set_frequency() {
    // NMIs need to be disabled

    let rate = 15; // 2 Hz

    let prev = read_register(REG_A);
    write_register(REG_A, (prev & 0b11110000) | rate);
}

pub(super) fn handle_interrupt() {
    trace!("tick");

    eoi();
}

fn eoi() {
    read_register(REG_C);
}
