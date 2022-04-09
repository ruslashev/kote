// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;

#[derive(Debug)]
pub struct PhysAddr(pub usize);

#[derive(Debug)]
pub struct VirtAddr(pub usize);

impl VirtAddr {
    pub fn from_u64(raw: u64) -> Self {
        Self(raw.try_into().expect("VirtAddr: u64 overflows usize"))
    }

    pub fn from_u32(raw: u32) -> Self {
        Self(raw.try_into().expect("VirtAddr: u32 overflows usize"))
    }
}

impl fmt::Display for PhysAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x?}", &self)
    }
}

impl fmt::Display for VirtAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x?}", &self)
    }
}
