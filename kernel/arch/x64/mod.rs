// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::arch::asm;
use core::{mem, ptr};

use crate::mm::types::{RootPageDirOps, VirtAddr};
use crate::process::Process;

#[macro_use]
pub mod asm;
pub mod backtrace;
pub mod interrupts;
pub mod mmu;
pub mod uart;

pub const KERNEL_BASE: usize = 0xffffff8000000000;

pub const USER_STACK_START: VirtAddr = VirtAddr(0x0000001000000000);
pub const USER_STACK_SIZE: usize = 4 * mmu::PAGE_SIZE;

pub const EMPTY_ROOT_DIR: RootPageDir = mmu::PageMapLevel4::empty();

pub type RegisterFrame = interrupts::exceptions::ExceptionFrame;
pub type RootPageDir = mmu::PageMapLevel4;
pub type LeafDirEntry = mmu::PageTableEntry;
pub type LeafDirEntryLarge = mmu::PageDirectoryEntry;

pub(self) const GDT_KERN_CODE: u16 = 8;
pub(self) const GDT_KERN_DATA: u16 = 16;
pub(self) const GDT_USER_DATA: u16 = 24;
pub(self) const GDT_USER_CODE: u16 = 32;
pub(self) const GDT_TSS_LOW: u16 = 40;
pub(self) const GDT_TSS_TOP: u16 = 48;

static mut TSS: TaskStateSegment = TaskStateSegment::new();

#[repr(C, packed)]
struct TaskStateSegment {
    res1: u32,
    rsp: [u64; 3],
    res2: u64,
    ist: [u64; 7],
    res3: u64,
    res4: u16,
    iomap_offs: u16,
}

impl TaskStateSegment {
    const fn new() -> Self {
        Self {
            res1: 0,
            rsp: [0; 3],
            res2: 0,
            ist: [0; 7],
            res3: 0,
            res4: 0,
            iomap_offs: mem::size_of::<TaskStateSegment>() as u16,
        }
    }
}

pub fn init() {
    extern "C" {
        fn int_stack_botmost();
        fn priv_stack_bot();
    }

    unsafe {
        TSS.ist[0] = int_stack_botmost as usize as u64;
        TSS.rsp[0] = priv_stack_bot as usize as u64;

        load_tss(&TSS);
    }

    set_syscall_msrs();
}

fn load_tss(tss: &TaskStateSegment) {
    extern "C" {
        fn gdt();
    }

    let addr = ptr::addr_of!(*tss) as u64;
    let size = mem::size_of::<TaskStateSegment>() as u64;

    let (low, top) = create_tss_descriptors(addr, size);

    let gdt_ptr = gdt as *mut u64;

    let tss_low_idx = GDT_TSS_LOW as usize / mem::size_of::<u64>();
    let tss_top_idx = GDT_TSS_TOP as usize / mem::size_of::<u64>();

    unsafe {
        gdt_ptr.add(tss_low_idx).write(low);
        gdt_ptr.add(tss_top_idx).write(top);

        asm!("ltr ax",
            in("ax") GDT_TSS_LOW);
    }
}

fn create_tss_descriptors(addr: u64, size: u64) -> (u64, u64) {
    let addr_1 = (addr & 0x000000000000ffff) >> 0;
    let addr_2 = (addr & 0x0000000000ff0000) >> 16;
    let addr_3 = (addr & 0x00000000ff000000) >> 24;
    let addr_4 = (addr & 0xffffffff00000000) >> 32;

    let size_low = (size & 0x0ffff) >> 0;
    let size_top = (size & 0xf0000) >> 16;

    let desc_type = 0b1001; // Available 64-bit TSS
    let privilege = 0;
    let present = 1;

    let low = (size_low << 0)
        | (addr_1 << 16)
        | (addr_2 << 32)
        | (desc_type << 40)
        | (privilege << 45)
        | (present << 47)
        | (size_top << 48)
        | (addr_3 << 56);

    let top = addr_4;

    (low, top)
}

fn set_syscall_msrs() {
    /*  STAR[47:32] AND 0xfffc = Kernel code
     *  STAR[47:32] + 8        = Kernel data
     *  STAR[63:48] + 16       = User code
     * (STAR[63:48] + 8) OR 3  = User data
     */

    let star_msr = 0xc000_0081;

    let star_low = GDT_KERN_CODE as u64;
    let star_top = (GDT_USER_DATA as u64 - 8) | 3;

    assert!(star_low & 0xfffc == GDT_KERN_CODE.into());
    assert!(star_low + 8 == GDT_KERN_DATA.into());

    assert!((star_top & 0xfffc) + 16 == GDT_USER_CODE.into());
    assert!((star_top + 8) | 3 == (GDT_USER_DATA | 3).into());

    let star = (star_top << 48) | (star_low << 32);

    asm::wrmsr(star_msr, star);

    let lstar_msr = 0xc000_0082;

    extern "C" {
        fn syscall_handler();
    }
    asm::wrmsr(lstar_msr, syscall_handler as usize as u64);

    let sfmask_msr = 0xc000_0084;
    let intr_flag = 1 << 9;

    asm::wrmsr(sfmask_msr, intr_flag);
}

pub fn switch_to_process(proc: Process) -> ! {
    interrupts::disable();

    proc.root_dir.switch_to_this();

    do_switch(&proc.registers);
}

#[naked]
extern "C" fn do_switch(registers: &RegisterFrame) -> ! {
    unsafe {
        asm!(r#"
        push [rdi + {offset_ss}]      // prepare data to be restored by iret
        push [rdi + {offset_rsp}]     // aka Long-Mode stack after interrupt
        push [rdi + {offset_rflags}]
        push [rdi + {offset_cs}]
        push [rdi + {offset_rip}]
        mov r15, [rdi + {offset_r15}] // restore rest of the registers
        mov r14, [rdi + {offset_r14}]
        mov r13, [rdi + {offset_r13}]
        mov r12, [rdi + {offset_r12}]
        mov r11, [rdi + {offset_r11}]
        mov r10, [rdi + {offset_r10}]
        mov r9,  [rdi + {offset_r9}]
        mov r8,  [rdi + {offset_r8}]
        mov rbp, [rdi + {offset_rbp}]
        mov rsi, [rdi + {offset_rsi}]
        mov rdx, [rdi + {offset_rdx}]
        mov rcx, [rdi + {offset_rcx}]
        mov rbx, [rdi + {offset_rbx}]
        mov rax, [rdi + {offset_rax}]
        mov rdi, [rdi + {offset_rdi}]
        iretq
        "#,
        // offset_of is still underway
        offset_r15 = const 0,
        offset_r14 = const 8,
        offset_r13 = const 16,
        offset_r12 = const 24,
        offset_r11 = const 32,
        offset_r10 = const 40,
        offset_r9 = const 48,
        offset_r8 = const 56,
        offset_rbp = const 64,
        offset_rdi = const 72,
        offset_rsi = const 80,
        offset_rdx = const 88,
        offset_rcx = const 96,
        offset_rbx = const 104,
        offset_rax = const 112,
        offset_rip = const 128,
        offset_cs = const 136,
        offset_rflags = const 144,
        offset_rsp = const 152,
        offset_ss = const 160,
        options(noreturn)
        );
    }
}
