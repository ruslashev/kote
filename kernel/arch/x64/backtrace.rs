// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

pub struct Backtrace {
    rbp: u64,
}

impl Backtrace {
    pub fn from_rbp(rbp: u64) -> Self {
        Backtrace { rbp }
    }

    pub fn from_here() -> Self {
        Backtrace { rbp: read_fp!() }
    }
}

impl Iterator for Backtrace {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        if self.rbp == 0 {
            return None;
        }

        let saved_rip = self.rbp + 8;
        let retaddr = unsafe { *(saved_rip as *const u64) };

        self.rbp = unsafe { *(self.rbp as *const u64) };

        if self.rbp == 0 {
            None
        } else {
            Some(retaddr)
        }
    }
}
