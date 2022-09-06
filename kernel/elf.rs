// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::mem::size_of;

use crate::arch::mmu;
use crate::mm;
use crate::mm::types::{Address, RegisterFrameOps, RootPageDirOps, VirtAddr};
use crate::process::Process;

type Elf64Addr = u64;
type Elf64Off = u64;
type Elf64Half = u16;
type Elf64Word = u32;
type Elf64Xword = u64;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Elf64Ehdr {
    pub e_ident: [u8; 16],
    pub e_type: Elf64Half,
    pub e_machine: Elf64Half,
    pub e_version: Elf64Word,
    pub e_entry: Elf64Addr,
    pub e_phoff: Elf64Off,
    pub e_shoff: Elf64Off,
    pub e_flags: Elf64Word,
    pub e_ehsize: Elf64Half,
    pub e_phentsize: Elf64Half,
    pub e_phnum: Elf64Half,
    pub e_shentsize: Elf64Half,
    pub e_shnum: Elf64Half,
    pub e_shstrndx: Elf64Half,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Elf64Phdr {
    pub p_type: Elf64Word,
    pub p_flags: Elf64Word,
    pub p_offset: Elf64Off,
    pub p_vaddr: Elf64Addr,
    pub p_paddr: Elf64Addr,
    pub p_filesz: Elf64Xword,
    pub p_memsz: Elf64Xword,
    pub p_align: Elf64Xword,
}

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct Elf64Shdr {
    pub sh_name: Elf64Word,
    pub sh_type: Elf64Word,
    pub sh_flags: Elf64Xword,
    pub sh_addr: Elf64Addr,
    pub sh_offset: Elf64Off,
    pub sh_size: Elf64Xword,
    pub sh_link: Elf64Word,
    pub sh_info: Elf64Word,
    pub sh_addralign: Elf64Xword,
    pub sh_entsize: Elf64Xword,
}

const EI_NIDENT: usize = 16;
const ET_EXEC: Elf64Half = 2;
const EM_X86_64: Elf64Half = 62;
const EV_CURRENT: Elf64Word = 1;

const PT_LOAD: Elf64Word = 1;
const PF_X: Elf64Word = 0b001;
const PF_W: Elf64Word = 0b010;
const PF_R: Elf64Word = 0b100;

macro_rules! read_int {
    ($ty:ident, $in:expr) => {{
        let (int_bytes, rest) = $in.split_at(size_of::<$ty>());
        $in = rest;
        $ty::from_le_bytes(int_bytes.try_into().unwrap())
    }};
}

macro_rules! read_last {
    ($ty:ident, $in:expr) => {{
        let ret = read_int!($ty, $in);
        _ = $in;
        ret
    }};
}

macro_rules! check_field {
    ($var:ident, $expected:expr) => {{
        assert!(
            $var == $expected,
            "bad {} ({}), expected {} ({})",
            stringify!($var),
            $var,
            stringify!($expected),
            $expected
        )
    }};
}

pub fn load(process: &mut Process, elf: &[u8]) {
    assert!(elf.len() > size_of::<Elf64Ehdr>(), "bad header length");
    assert!(&elf[0..4] == b"\x7fELF", "bad magic");

    let mut input = &elf[EI_NIDENT..];

    let e_type = read_int!(Elf64Half, input);
    let e_machine = read_int!(Elf64Half, input);
    let e_version = read_int!(Elf64Word, input);
    let e_entry = read_int!(Elf64Addr, input);
    let e_phoff = read_int!(Elf64Off, input);
    let _e_shoff = read_int!(Elf64Off, input);
    let _e_flags = read_int!(Elf64Word, input);
    let _e_ehsize = read_int!(Elf64Half, input);
    let e_phentsize = read_int!(Elf64Half, input);
    let e_phnum = read_int!(Elf64Half, input);
    let _e_shentsize = read_int!(Elf64Half, input);
    let _e_shnum = read_int!(Elf64Half, input);
    let _e_shstrndx = read_last!(Elf64Half, input);

    check_field!(e_type, ET_EXEC);
    check_field!(e_machine, EM_X86_64);
    check_field!(e_version, EV_CURRENT);
    check_field!(e_phentsize, size_of::<Elf64Phdr>() as u16);

    let mut phdrs = &elf[e_phoff as usize..];
    for _ in 0..e_phnum {
        load_program_header(process, &mut phdrs, elf);
    }

    process.registers.set_program_counter(e_entry as usize);
}

fn load_program_header(process: &mut Process, input: &mut &[u8], elf: &[u8]) {
    let p_type = read_int!(Elf64Word, *input);
    let p_flags = read_int!(Elf64Word, *input);
    let p_offset = read_int!(Elf64Off, *input);
    let p_vaddr = read_int!(Elf64Addr, *input);
    let _p_paddr = read_int!(Elf64Addr, *input);
    let p_filesz = read_int!(Elf64Xword, *input);
    let p_memsz = read_int!(Elf64Xword, *input);
    let _p_align = read_int!(Elf64Xword, *input);

    if p_type != PT_LOAD {
        return;
    }

    let vaddr = VirtAddr::from_u64(p_vaddr);
    let size_in_mem = p_memsz as usize;
    let slice = unsafe { vaddr.into_slice_mut(size_in_mem) };
    let file_pos = p_offset as usize;
    let file_len = p_filesz as usize;

    process.root_dir.alloc_range(vaddr, size_in_mem, mmu::USER_ACCESSIBLE | mmu::WRITABLE);

    process.root_dir.switch_to_this();

    assert!(file_len <= size_in_mem);
    slice.copy_from_slice(&elf[file_pos..file_pos + file_len]);

    mm::switch_to_kernel_root_dir();

    slice[file_len..size_in_mem].fill(0);

    if p_flags & PF_W == 0 {
        process.root_dir.change_range_perms(
            vaddr,
            size_in_mem,
            mmu::USER_ACCESSIBLE | mmu::PRESENT,
        );
    }
}
