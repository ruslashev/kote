// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use super::exceptions::ExceptionFrame;
use super::rtc;
use crate::arch::asm::io;
use crate::sched;

const PIC_IRQ_OFFSET: u8 = 32;
const PIC1: u16 = 0x20;
const PIC2: u16 = 0xa0;
const PIC1_DATA: u16 = PIC1 + 1;
const PIC2_DATA: u16 = PIC2 + 1;

/// Remap IRQs from 0..15 to 32..47 (+`PIC_IRQ_OFFSET`) to not conflict with CPU exceptions
pub fn remap() {
    // Begin initialization
    outb_wait(PIC1, 0b00010001);
    outb_wait(PIC2, 0b00010001);

    // Tell offsets
    outb_wait(PIC1_DATA, PIC_IRQ_OFFSET);
    outb_wait(PIC2_DATA, PIC_IRQ_OFFSET + 8);

    // Setup cascading
    outb_wait(PIC1_DATA, 4);
    outb_wait(PIC2_DATA, 2);

    // Set 8086 mode
    outb_wait(PIC1_DATA, 1);
    outb_wait(PIC2_DATA, 1);

    // Disable all lines
    outb_wait(PIC1_DATA, 0xff);
    outb_wait(PIC2_DATA, 0xff);
}

#[inline(always)]
fn outb_wait(port: u16, val: u8) {
    io::outb(port, val);
    io::wait();
}

pub fn enable_line(irq: u8) {
    let port;
    let line;

    if irq < 8 {
        port = PIC1_DATA;
        line = irq;
    } else {
        port = PIC2_DATA;
        line = irq - 8;
    }

    let bit = 1 << line;
    let value = io::inb(port) & !bit;

    io::outb(port, value);
}

pub fn disable_line(irq: u8) {
    let port;
    let line;

    if irq < 8 {
        port = PIC1_DATA;
        line = irq;
    } else {
        port = PIC2_DATA;
        line = irq - 8;
    }

    let bit = 1 << line;
    let value = io::inb(port) | bit;

    io::outb(port, value);
}

/// Send end-of-interrupt
fn irq_eoi(irq: u8) {
    let eoi = 0x20;

    if irq >= 8 {
        io::outb(PIC2, eoi);
    }

    io::outb(PIC1, eoi);
}

#[no_mangle]
pub extern "C" fn irq_dispatch(frame: &ExceptionFrame) {
    let vec = frame.number as u8;

    sched::current().registers = *frame;

    if vec == 8 {
        rtc::handle_interrupt();
        irq_eoi(vec);
        sched::next();
    }

    irq_eoi(vec);
}
