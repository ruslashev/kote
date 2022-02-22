// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::arch::asm;

use super::io;

static mut IDT: [IDTEntry; 256] = [IDTEntry {
    handler_addr_low: 0,
    gdt_selector: 0,
    attributes: 0,
    handler_addr_mid: 0,
    handler_addr_top: 0,
    reserved: 0,
}; 256];

static EXCEPTION_HANDLERS: [Exception; 32] = [
    Exception::with_hdl("Divide-by-zero Error", divide_by_zero), // 0
    Exception::stub_hdl("Debug"),                                // 1
    Exception::stub_hdl("Non-maskable Interrupt"),               // 2
    Exception::stub_hdl("Breakpoint"),                           // 3
    Exception::stub_hdl("Overflow"),                             // 4
    Exception::stub_hdl("Bound Range Exceeded"),                 // 5
    Exception::stub_hdl("Invalid Opcode"),                       // 6
    Exception::stub_hdl("Device Not Available"),                 // 7
    Exception::stub_hdl("Double Fault"),                         // 8
    Exception::stub_hdl("Coprocessor Segment Overrun"),          // 9
    Exception::stub_hdl("Invalid TSS"),                          // 10
    Exception::stub_hdl("Segment Not Present"),                  // 11
    Exception::stub_hdl("Stack-Segment Fault"),                  // 12
    Exception::stub_hdl("General Protection Fault"),             // 13
    Exception::stub_hdl("Page Fault"),                           // 14
    Exception::reserved(),                                       // 15
    Exception::stub_hdl("x87 Floating-Point Exception"),         // 16
    Exception::stub_hdl("Alignment Check"),                      // 17
    Exception::stub_hdl("Machine Check"),                        // 18
    Exception::stub_hdl("SIMD Floating-Point Exception"),        // 19
    Exception::stub_hdl("Virtualization Exception"),             // 20
    Exception::stub_hdl("Control Protection Exception"),         // 21
    Exception::reserved(),                                       // 22
    Exception::reserved(),                                       // 23
    Exception::reserved(),                                       // 24
    Exception::reserved(),                                       // 25
    Exception::reserved(),                                       // 26
    Exception::reserved(),                                       // 27
    Exception::stub_hdl("Hypervisor Injection Exception"),       // 28
    Exception::stub_hdl("VMM Communication Exception"),          // 29
    Exception::stub_hdl("Security Exception"),                   // 30
    Exception::reserved(),                                       // 31
];

const PIC_IRQ_OFFSET: u8 = 32;
const PIC1: u16 = 0x20;
const PIC2: u16 = 0xa0;

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

struct Exception {
    name: &'static str,
    handler: Option<fn()>,
}

impl Exception {
    const fn with_hdl(name: &'static str, handler: fn()) -> Self {
        Exception {
            name,
            handler: Some(handler),
        }
    }

    const fn stub_hdl(name: &'static str) -> Self {
        Exception {
            name,
            handler: None,
        }
    }

    const fn reserved() -> Self {
        Exception {
            name: "Reserved",
            handler: None,
        }
    }
}

#[inline(always)]
pub fn enable() {
    unsafe {
        asm!("sti", options(nostack));
    }
}

#[inline(always)]
pub fn disable() {
    unsafe {
        asm!("cli", options(nostack));
    }
}

pub fn init() {
    remap_pic();

    build_idt();
    load_idt();
}

/// Remap IRQs from 0..15 to 32..47 (+PIC_IRQ_OFFSET) to not conflict with CPU exceptions
fn remap_pic() {
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
fn irq_eoi(irq: u8) {
    let eoi = 0x20;

    if irq >= 8 {
        io::outb(PIC2, eoi);
    }

    io::outb(PIC1, eoi);
}

// Can't use statics in const fns, otherwise this would've been one
fn build_idt() {
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
        fn handle_irq_3();
        fn handle_irq_4();
        fn handle_irq_5();
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
        IDT[35] = create_idt_entry(handle_irq_3, true, false);
        IDT[36] = create_idt_entry(handle_irq_4, true, false);
        IDT[37] = create_idt_entry(handle_irq_5, true, false);
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

fn load_idt() {
    #[repr(C, packed)]
    struct IDTDescriptor {
        size: u16,
        addr: u64,
    }

    unsafe {
        let idtr = IDTDescriptor {
            size: 256 * core::mem::size_of::<IDTEntry>() as u16 - 1,
            addr: &IDT as *const _ as u64,
        };

        asm!("lidt [{}]",
            in(reg) &idtr,
            options(nostack));
    }
}

#[no_mangle]
pub extern "C" fn exception_dispatch(rsp: u64) {
    /* From AMD64 Architecture Programmer’s Manual, Volume 2: System Programming - 24593,
     * Figure 8-14, Long-Mode Stack After Interrupt -- Same Privilege,
     * modified with additional registers pushed in interrupts.s:
     *
     * ┌───────────────────────┬────────┐
     * │                       │ ret.SS │
     * ├───────────────────────┴────────┤
     * │           return RSP           │
     * ├────────────────────────────────┤
     * │           return RFLAGS        │
     * ├───────────────────────┬────────┤
     * │                       │ ret.CS │
     * ├───────────────────────┴────────┤
     * │           return RIP           │
     * ├───────────────┬────────────────┤
     * │  exc. vector  │ error code (0) │
     * ├───────────────┴────────────────┤
     * │              rax               │
     * ├────────────────────────────────┤
     * │              rbx               │
     * ├────────────────────────────────┤
     * │              rcx               │
     * ├────────────────────────────────┤
     * │              rdx               │
     * ├────────────────────────────────┤
     * │              rsi               │
     * ├────────────────────────────────┤
     * │              rdi               │
     * ├────────────────────────────────┤
     * │              rbp               │
     * ├────────────────────────────────┤
     * │              r8                │
     * ├────────────────────────────────┤
     * │              r9                │
     * ├────────────────────────────────┤
     * │              r10               │
     * ├────────────────────────────────┤
     * │              r11               │
     * ├────────────────────────────────┤
     * │              r12               │
     * ├────────────────────────────────┤
     * │              r13               │ +16
     * ├────────────────────────────────┤
     * │              r14               │ +8
     * ├────────────────────────────────┤
     * │              r15               │ <- New RSP
     * └────────────────────────────────┘
     */

    #[repr(C, packed)]
    struct ExceptionFrame {
        r15: u64,
        r14: u64,
        r13: u64,
        r12: u64,
        r11: u64,
        r10: u64,
        r9: u64,
        r8: u64,
        rbp: u64,
        rdi: u64,
        rsi: u64,
        rdx: u64,
        rcx: u64,
        rbx: u64,
        rax: u64,
        error_code: u32,
        exc_vector: u32,
        return_rip: u64,
        return_cs: u64,
        rflags: u64,
        return_rsp: u64,
        return_ss: u64,
    }

    let frame = rsp as *const ExceptionFrame;
    let vec = unsafe { (*frame).exc_vector };
    let exc_handler = &EXCEPTION_HANDLERS[vec as usize];

    println!("Exception {} occured: {}", vec, exc_handler.name);

    if let Some(handler) = exc_handler.handler {
        handler.call(());
    }

    loop {}
}

#[no_mangle]
pub extern "C" fn irq_dispatch(vec: u8) {
    println!("In IRQ {} handler", vec);

    irq_eoi(vec);
}

fn divide_by_zero() {
    println!("Divide by zero handler");
}
