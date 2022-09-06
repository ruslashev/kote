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

const GDT_KERN_CODE: u16 = 8;
const GDT_KERN_DATA: u16 = 16;
const GDT_USER_DATA: u16 = 24;
const GDT_USER_CODE: u16 = 32;
const GDT_TSS_LOW: u16 = 40;
const GDT_TSS_TOP: u16 = 48;

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

    set_star_msr();
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

fn set_star_msr() {
    /*  STAR[47:32] AND 0xFFFC = Kernel code
     *  STAR[47:32] + 8        = Kernel data
     *  STAR[63:48] + 16       = User code
     * (STAR[63:48] + 8) OR 3  = User data
     */

    let star_msr = 0xc0000081;

    let star_low = GDT_KERN_CODE as u64;
    let star_top = (GDT_USER_DATA as u64 - 8) | 3;

    assert!(star_low & 0xfffc == GDT_KERN_CODE.into());
    assert!(star_low + 8 == GDT_KERN_DATA.into());

    assert!((star_top & 0xfffc) + 16 == GDT_USER_CODE.into());
    assert!((star_top + 8) | 3 == (GDT_USER_DATA | 3).into());

    let star = (star_top << 48) | (star_low << 32);

    asm::wrmsr(star_msr, star);
}

pub fn switch_to_process(proc: Process) {
    let rip = proc.registers.return_rip;
    let flags = proc.registers.rflags;
    let rsp = proc.registers.return_rsp;

    println!("switch_to_process: rip={:#x}, flags={:#b}, rsp={:#x}", rip, flags, rsp);

    let cs = GDT_USER_CODE | 3;
    let ds = GDT_USER_DATA | 3;

    proc.root_dir.switch_to_this();

    unsafe {
        asm!(
            "mov ds, ax",
            "push rax",
            "push {0:r}",
            "push {1:r}",
            "push {2:r}",
            "push {3:r}",
            "iretq",
            in(reg) rsp,
            in(reg) flags,
            in(reg) cs,
            in(reg) rip,
            in("rax") ds,
        );
    }
}
