// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;

use super::handlers;
use crate::arch::backtrace::Backtrace;

static EXCEPTION_HANDLERS: [Exception; 32] = [
    Exception::with_hdl("Divide-by-zero Error", handlers::divide_by_zero), // 0
    Exception::stub_hdl("Debug"),                                          // 1
    Exception::stub_hdl("Non-maskable Interrupt"),                         // 2
    Exception::stub_hdl("Breakpoint"),                                     // 3
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
    Exception::stub_hdl("Page Fault"),                                     // 14
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

impl fmt::Display for ExceptionFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Copies to avoid references to packed fields
        let rip = self.return_rip;
        let rdi = self.rdi;
        let rsi = self.rsi;
        let rdx = self.rdx;
        let rcx = self.rcx;
        let r8 = self.r8;
        let r9 = self.r9;
        let rsp = self.return_rsp;
        let rbp = self.rbp;

        writeln!(f, "RIP {:#x}", rip)?;
        writeln!(f, "RDI {:#x}", rdi)?;
        writeln!(f, "RSI {:#x}", rsi)?;
        writeln!(f, "RDX {:#x}", rdx)?;
        writeln!(f, "RCX {:#x}", rcx)?;
        writeln!(f, "R8  {:#x}", r8)?;
        writeln!(f, "R9  {:#x}", r9)?;
        writeln!(f, "RSP {:#x}", rsp)?;
        writeln!(f, "RBP {:#x}", rbp)?;

        let flags = self.rflags;

        writeln!(f, "Flags {:#b}", flags)?;

        writeln!(f, "Backtrace:")?;
        writeln!(f, " 1) {:#x}", rip)?;

        let mut counter = 2;
        for addr in Backtrace::from_rbp(rbp) {
            writeln!(f, "{:>2}) {:#x}", counter, addr)?;
            counter += 1;
        }

        Ok(())
    }
}

#[no_mangle]
pub extern "C" fn exception_dispatch(rsp: u64) {
    let frame_ptr = rsp as *const ExceptionFrame;
    let frame = unsafe { &*frame_ptr };
    let vec = frame.exc_vector;
    let exc_handler = &EXCEPTION_HANDLERS[vec as usize];

    println!("Exception {} occured: {}", vec, exc_handler.name);
    println!("{}", frame);

    if let Some(handler) = exc_handler.handler {
        handler.call(());
    }

    loop {}
}
