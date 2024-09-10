// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;

use super::handlers;
use crate::arch::backtrace::Backtrace;
use crate::{arch, mm, sched};

static EXCEPTION_HANDLERS: [Exception; 32] = [
    Exception::with_hdl("Divide-by-zero Error", handlers::divide_by_zero), // 0
    Exception::stub_hdl("Debug"),                                          // 1
    Exception::stub_hdl("Non-maskable Interrupt"),                         // 2
    Exception::with_hdl("Breakpoint", handlers::breakpoint),               // 3
    Exception::stub_hdl("Overflow"),                                       // 4
    Exception::stub_hdl("Bound Range Exceeded"),                           // 5
    Exception::stub_hdl("Invalid Opcode"),                                 // 6
    Exception::stub_hdl("Device Not Available"),                           // 7
    Exception::stub_hdl("Double Fault"),                                   // 8
    Exception::stub_hdl("Coprocessor Segment Overrun"),                    // 9
    Exception::stub_hdl("Invalid TSS"),                                    // 10
    Exception::stub_hdl("Segment Not Present"),                            // 11
    Exception::stub_hdl("Stack-Segment Fault"),                            // 12
    Exception::stub_hdl("General Protection Fault"),                       // 13
    Exception::with_hdl("Page Fault", handlers::page_fault),               // 14
    Exception::reserved(),                                                 // 15
    Exception::stub_hdl("x87 Floating-Point Exception"),                   // 16
    Exception::stub_hdl("Alignment Check"),                                // 17
    Exception::stub_hdl("Machine Check"),                                  // 18
    Exception::stub_hdl("SIMD Floating-Point Exception"),                  // 19
    Exception::stub_hdl("Virtualization Exception"),                       // 20
    Exception::stub_hdl("Control Protection Exception"),                   // 21
    Exception::reserved(),                                                 // 22
    Exception::reserved(),                                                 // 23
    Exception::reserved(),                                                 // 24
    Exception::reserved(),                                                 // 25
    Exception::reserved(),                                                 // 26
    Exception::reserved(),                                                 // 27
    Exception::stub_hdl("Hypervisor Injection Exception"),                 // 28
    Exception::stub_hdl("VMM Communication Exception"),                    // 29
    Exception::stub_hdl("Security Exception"),                             // 30
    Exception::reserved(),                                                 // 31
];

struct Exception {
    name: &'static str,
    handler: Option<fn(&ExceptionFrame)>,
}

impl Exception {
    const fn with_hdl(name: &'static str, handler: fn(&ExceptionFrame)) -> Self {
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
 * │  exc/irq num  │ error code (0) │
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
#[derive(Debug, Default, Clone, Copy)]
#[repr(C, packed)]
pub struct ExceptionFrame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub error_code: u32,
    pub number: u32,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

impl fmt::Display for ExceptionFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let rip = self.rip;

        writeln!(
            f,
            "RIP {:<#18x} RSP {:<#18x} Err {:<#18x} Flags {:#022b}",
            rip,
            { self.rsp },
            { self.error_code },
            { self.rflags },
        )?;
        writeln!(
            f,
            "RAX {:<#18x} RBX {:<#18x} RCX {:<#18x} RDX {:<#18x} RSI {:<#18x}",
            { self.rax },
            { self.rbx },
            { self.rcx },
            { self.rdx },
            { self.rsi },
        )?;
        writeln!(
            f,
            "RDI {:<#18x} RBP {:<#18x} R8  {:<#18x} R9  {:<#18x} R10 {:<#18x}",
            { self.rdi },
            { self.rbp },
            { self.r8 },
            { self.r9 },
            { self.r10 },
        )?;
        writeln!(
            f,
            "R11 {:<#18x} R12 {:<#18x} R13 {:<#18x} R14 {:<#18x} R15 {:<#18x}",
            { self.r11 },
            { self.r12 },
            { self.r13 },
            { self.r14 },
            { self.r15 },
        )?;

        writeln!(f, "Backtrace:")?;

        let mut backtrace = Backtrace::from_rbp(self.rbp).enumerate().peekable();

        // Last write statement must not include newline
        if backtrace.peek().is_none() {
            write!(f, " 1) {:#x}", rip)?;
        } else {
            writeln!(f, " 1) {:#x}", rip)?;

            while let Some((i, addr)) = backtrace.next() {
                if backtrace.peek().is_some() {
                    writeln!(f, "{:>2}) {:#x}", i + 2, addr)?;
                } else {
                    write!(f, "{:>2}) {:#x}", i + 2, addr)?;
                }
            }
        }

        Ok(())
    }
}

impl mm::types::RegisterFrameOps for ExceptionFrame {
    fn new_userspace() -> Self {
        let stack_top = arch::USER_STACK_START.0 + arch::USER_STACK_SIZE - 16;
        let intr_flag = 1 << 9;

        Self {
            rsp: stack_top as u64,
            rflags: intr_flag,
            cs: (arch::GDT_USER_CODE | 3).into(),
            ss: (arch::GDT_USER_DATA | 3).into(),

            ..Default::default()
        }
    }

    fn set_program_counter(&mut self, addr: usize) {
        self.rip = addr as u64;
    }
}

#[no_mangle]
pub extern "C" fn exception_dispatch(frame: &ExceptionFrame) {
    let vec = frame.number;
    let exc_handler = &EXCEPTION_HANDLERS[vec as usize];

    sched::current().registers = *frame;

    println!("Exception {} occured: {}", vec, exc_handler.name);
    println!("{}", frame);

    if let Some(handler) = exc_handler.handler {
        handler.call((frame,));
    }

    loop {}
}
