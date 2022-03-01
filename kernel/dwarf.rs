// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::sync::atomic::{AtomicUsize, Ordering};
use core::{slice, str};

use crate::bootloader::BootloaderInfo;

struct State {
    address: usize,
    file: usize,
    line: usize,
    column: usize,
    end_seq: bool,
    isa: usize,
    discr: usize,
}

struct LineNumHeader {
    min_instr_length: u8,
    max_ops_per_inst: u8,
    line_base: i8,
    line_range: u8,
    opcode_base: u8,
    std_opcode_lengths: *const u8,
}

pub struct Info {
    pub line: usize,
}

// Really gets the noggin joggin
#[derive(Default)]
struct SyncSlice {
    addr: AtomicUsize,
    size: AtomicUsize,
}

static SECT_DEBUG_LINES: SyncSlice = SyncSlice::default();

impl Default for State {
    fn default() -> Self {
        State {
            address: 0,
            file: 1,
            line: 1,
            column: 0,
            end_seq: false,
            isa: 0,
            discr: 0,
        }
    }
}

unsafe impl Sync for SyncSlice {}

impl SyncSlice {
    const fn default() -> Self {
        SyncSlice {
            addr: AtomicUsize::new(0),
            size: AtomicUsize::new(0),
        }
    }

    fn set(&self, addr: usize, size: usize) {
        self.addr.store(addr, Ordering::Relaxed);
        self.size.store(size, Ordering::Relaxed);
    }

    fn to_slice(&self) -> &[u8] {
        let addr = self.addr.load(Ordering::Relaxed) as *const u8;
        let size = self.size.load(Ordering::Relaxed);
        unsafe { slice::from_raw_parts(addr, size) }
    }
}

pub fn init(info: &BootloaderInfo) {
    if info.section_headers.is_none() {
        return;
    }

    let shdr_info = info.section_headers.as_ref().unwrap();
    let entries = shdr_info.shdrs;

    let shstrtab = unsafe {
        let idx = shdr_info.shstrtab_idx;
        let shdr = entries.add(idx).read();
        let addr = shdr.sh_addr as *const u8;
        let size = shdr.sh_size as usize;

        slice::from_raw_parts(addr, size)
    };

    for entry in 0..shdr_info.num_shdrs {
        let shdr = unsafe { entries.add(entry).read() };
        let name_beg = shdr.sh_name as usize;
        let mut name_end = None;

        for (end_idx, &ch) in shstrtab.iter().enumerate().skip(name_beg) {
            if ch == 0 {
                name_end = Some(end_idx);
                break;
            }
        }

        assert!(name_end.is_some(), "DWARF: kernel shstrtab overflow");

        let name_end = name_end.unwrap();
        let name = str::from_utf8(&shstrtab[name_beg..name_end]).unwrap();

        if name == ".debug_line" {
            let addr = shdr.sh_addr as usize;
            let size = shdr.sh_size as usize;

            SECT_DEBUG_LINES.set(addr, size);
        }
    }
}

pub fn get_info_for_addr(_addr: usize) {}
