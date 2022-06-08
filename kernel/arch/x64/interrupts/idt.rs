// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::arch::asm;

static mut IDT: [IDTEntry; 256] = [IDTEntry {
    handler_addr_low: 0,
    gdt_selector: 0,
    attributes: 0,
    handler_addr_mid: 0,
    handler_addr_top: 0,
    reserved: 0,
}; 256];

#[derive(Copy, Clone)]
#[repr(C, packed)]
struct IDTEntry {
    handler_addr_low: u16,
    gdt_selector: u16,
    attributes: u16,
    handler_addr_mid: u16,
    handler_addr_top: u32,
    reserved: u32,
}

fn build() {
    // Can't use statics in const fns, otherwise this would've been one
    extern "C" {
        fn handle_exception_0();
        fn handle_exception_1();
        fn handle_exception_2();
        fn handle_exception_3();
        fn handle_exception_4();
        fn handle_exception_5();
        fn handle_exception_6();
        fn handle_exception_7();
        fn handle_exception_8();
        fn handle_exception_9();
        fn handle_exception_10();
        fn handle_exception_11();
        fn handle_exception_12();
        fn handle_exception_13();
        fn handle_exception_14();
        fn handle_exception_15();
        fn handle_exception_16();
        fn handle_exception_17();
        fn handle_exception_18();
        fn handle_exception_19();
        fn handle_exception_20();
        fn handle_exception_21();
        fn handle_exception_22();
        fn handle_exception_23();
        fn handle_exception_24();
        fn handle_exception_25();
        fn handle_exception_26();
        fn handle_exception_27();
        fn handle_exception_28();
        fn handle_exception_29();
        fn handle_exception_30();
        fn handle_exception_31();

        fn handle_irq_0();
        fn handle_irq_1();
        fn handle_irq_2();
        fn handle_irq_8();
    }

    unsafe {
        IDT[0] = create_idt_entry(handle_exception_0, true, true);
        IDT[1] = create_idt_entry(handle_exception_1, true, true);
        IDT[2] = create_idt_entry(handle_exception_2, true, true);
        IDT[3] = create_idt_entry(handle_exception_3, true, true);
        IDT[4] = create_idt_entry(handle_exception_4, true, true);
        IDT[5] = create_idt_entry(handle_exception_5, true, true);
        IDT[6] = create_idt_entry(handle_exception_6, true, true);
        IDT[7] = create_idt_entry(handle_exception_7, true, true);
        IDT[8] = create_idt_entry(handle_exception_8, true, true);
        IDT[9] = create_idt_entry(handle_exception_9, true, true);
        IDT[10] = create_idt_entry(handle_exception_10, true, true);
        IDT[11] = create_idt_entry(handle_exception_11, true, true);
        IDT[12] = create_idt_entry(handle_exception_12, true, true);
        IDT[13] = create_idt_entry(handle_exception_13, true, true);
        IDT[14] = create_idt_entry(handle_exception_14, true, true);
        IDT[15] = create_idt_entry(handle_exception_15, true, true);
        IDT[16] = create_idt_entry(handle_exception_16, true, true);
        IDT[17] = create_idt_entry(handle_exception_17, true, true);
        IDT[18] = create_idt_entry(handle_exception_18, true, true);
        IDT[19] = create_idt_entry(handle_exception_19, true, true);
        IDT[20] = create_idt_entry(handle_exception_20, true, true);
        IDT[21] = create_idt_entry(handle_exception_21, true, true);
        IDT[22] = create_idt_entry(handle_exception_22, true, true);
        IDT[23] = create_idt_entry(handle_exception_23, true, true);
        IDT[24] = create_idt_entry(handle_exception_24, true, true);
        IDT[25] = create_idt_entry(handle_exception_25, true, true);
        IDT[26] = create_idt_entry(handle_exception_26, true, true);
        IDT[27] = create_idt_entry(handle_exception_27, true, true);
        IDT[28] = create_idt_entry(handle_exception_28, true, true);
        IDT[29] = create_idt_entry(handle_exception_29, true, true);
        IDT[30] = create_idt_entry(handle_exception_30, true, true);
        IDT[31] = create_idt_entry(handle_exception_31, true, true);

        IDT[32] = create_idt_entry(handle_irq_0, true, false);
        IDT[33] = create_idt_entry(handle_irq_1, true, false);
        IDT[34] = create_idt_entry(handle_irq_2, true, false);
        IDT[40] = create_idt_entry(handle_irq_8, true, false);
    }
}

fn create_idt_entry(
    handler: unsafe extern "C" fn(),
    kernel_only: bool,
    is_exception: bool,
) -> IDTEntry {
    let handler_addr = handler as usize;
    let handler_addr_low = ((handler_addr & 0x000000000000ffff) >> 0) as u16;
    let handler_addr_mid = ((handler_addr & 0x00000000ffff0000) >> 16) as u16;
    let handler_addr_top = ((handler_addr & 0xffffffff00000000) >> 32) as u32;

    let present = 1;

    let ring = if kernel_only { 0 } else { 3 };

    let gate_type = if is_exception {
        // 64-bit trap gate
        0b1111
    } else {
        // 64-bit interrupt gate
        0b1110
    };

    let attributes: u16 = (present << 15) | (ring << 13) | (gate_type << 8);

    IDTEntry {
        handler_addr_low,
        gdt_selector: 0x8,
        attributes,
        handler_addr_mid,
        handler_addr_top,
        reserved: 0,
    }
}

fn load() {
    #[repr(C, packed)]
    struct IDTDescriptor {
        size: u16,
        addr: u64,
    }

    unsafe {
        let idtr = IDTDescriptor {
            size: 256 * core::mem::size_of::<IDTEntry>() as u16 - 1,
            addr: core::ptr::addr_of!(IDT) as u64,
        };

        asm!("lidt [{}]",
            in(reg) &idtr,
            options(nostack));
    }
}

pub(super) fn init() {
    build();
    load();
}
