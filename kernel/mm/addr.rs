// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;

#[derive(Copy, Clone, Debug)]
pub struct PhysAddr(pub usize);

#[derive(Copy, Clone, Debug)]
pub struct VirtAddr(pub usize);

pub trait Address: From<usize> {
    fn from_u64(raw: u64) -> Self {
        Self::from(raw.try_into().expect("Address: u64 overflows usize"))
    }

    fn from_u32(raw: u32) -> Self {
        Self::from(raw.try_into().expect("Address: u32 overflows usize"))
    }
}

impl From<usize> for PhysAddr {
    fn from(scalar: usize) -> Self {
        Self(scalar)
    }
}

impl From<usize> for VirtAddr {
    fn from(scalar: usize) -> Self {
        Self(scalar)
    }
}

impl Address for PhysAddr {}

impl Address for VirtAddr {}

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
