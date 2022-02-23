// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::arch::io;

const PIC_IRQ_OFFSET: u8 = 32;
const PIC1: u16 = 0x20;
const PIC2: u16 = 0xa0;

/// Remap IRQs from 0..15 to 32..47 (+PIC_IRQ_OFFSET) to not conflict with CPU exceptions
pub fn remap() {
    let pic1_data = PIC1 + 1;
    let pic2_data = PIC2 + 1;

    // Begin initialization
    outb_wait(PIC1, 0x11);
    outb_wait(PIC2, 0x11);

    // Tell offsets
    outb_wait(pic1_data, PIC_IRQ_OFFSET);
    outb_wait(pic2_data, PIC_IRQ_OFFSET + 8);

    // Setup cascading
    outb_wait(pic1_data, 4);
    outb_wait(pic2_data, 2);

    // Set 8086 mode for some reason
    outb_wait(pic1_data, 1);
    outb_wait(pic2_data, 1);

    // Clear masks
    outb_wait(pic1_data, 0);
    outb_wait(pic2_data, 0);
}

#[inline(always)]
fn outb_wait(port: u16, val: u8) {
    io::outb(port, val);
    io::io_wait();
}

/// Send end-of-interrupt
pub fn irq_eoi(irq: u8) {
    let eoi = 0x20;

    if irq >= 8 {
        io::outb(PIC2, eoi);
    }

    io::outb(PIC1, eoi);
}
