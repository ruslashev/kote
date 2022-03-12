// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::spinlock::SpinlockMutex;

/// Number of entries in a directory of any level (PML4, PDPT, PD, PT). Equal to 4096 B / 64 b.
const ENTRIES: usize = 512;

static ROOT_DIR: SpinlockMutex<PML4> = SpinlockMutex::new(PML4::empty());

struct PML4 {
    addr: u64,
    entries: &'static mut [PML4E],
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PML4E {
    scalar: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PDPE {
    scalar: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PDE {
    scalar: u64,
}

#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PTE {
    scalar: u64,
}

impl PML4 {
    const fn empty() -> Self {
        PML4 {
            addr: 0,
            entries: &mut [],
        }
    }

    fn from_addr(addr: u64) -> Self {
        let entries = unsafe {
            let addr = addr as *mut PML4E;
            core::slice::from_raw_parts_mut(addr, ENTRIES)
        };

        Self {
            addr,
            entries,
        }
    }

    fn clear(&mut self) {
        let zero_entry = PML4E { scalar: 0 };

        self.entries.fill(zero_entry);
    }
}

pub fn init() {
    // Defined in start.s
    extern "C" {
        static pml4: u64;
    }

    *ROOT_DIR.guard().data = PML4::from_addr(unsafe { pml4 });
}
