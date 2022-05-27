// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::fmt;

use crate::arch::mmu::{PAGE_SIZE, PAGE_SIZE_LARGE};

pub trait PowerOfTwoOps: Copy {
    fn is_po2_aligned(self, po2: Self) -> bool;
    fn po2_round_down(self, po2: Self) -> Self;
    fn po2_round_up(self, po2: Self) -> Self;

    fn is_page_aligned(self) -> bool;
    fn page_round_down(self) -> Self;
    fn page_round_up(self) -> Self;

    fn is_lpage_aligned(self) -> bool;
    fn lpage_round_down(self) -> Self;
    fn lpage_round_up(self) -> Self;
}

macro_rules! impl_po2_ops {
    ( $( $type:ident )+ ) => {
        $(
impl PowerOfTwoOps for $type {
    fn is_po2_aligned(self, po2: Self) -> bool {
        self & (po2 - 1) == 0
    }
    fn po2_round_down(self, po2: Self) -> Self {
        self & !(po2 - 1)
    }
    fn po2_round_up(self, po2: Self) -> Self {
        (self + po2 - 1) & !(po2 - 1)
    }

    fn is_page_aligned(self) -> bool {
        self.is_po2_aligned(PAGE_SIZE as Self)
    }
    fn page_round_down(self) -> Self {
        self.po2_round_down(PAGE_SIZE as Self)
    }
    fn page_round_up(self) -> Self {
        self.po2_round_up(PAGE_SIZE as Self)
    }

    fn is_lpage_aligned(self) -> bool {
        self.is_po2_aligned(PAGE_SIZE_LARGE as Self)
    }
    fn lpage_round_down(self) -> Self {
        self.po2_round_down(PAGE_SIZE_LARGE as Self)
    }
    fn lpage_round_up(self) -> Self {
        self.po2_round_up(PAGE_SIZE_LARGE as Self)
    }
}
        )*
    }
}

impl_po2_ops!(u32 u64 usize);

pub trait Bytes: Sized {
    fn to_bytes(self) -> usize;
}

pub struct KiB(pub usize);

pub struct MiB(pub usize);

pub struct GiB(pub usize);

impl const Bytes for KiB {
    fn to_bytes(self) -> usize {
        self.0 * 1024
    }
}

impl const Bytes for MiB {
    fn to_bytes(self) -> usize {
        self.0 * 1024 * 1024
    }
}

impl const Bytes for GiB {
    fn to_bytes(self) -> usize {
        self.0 * 1024 * 1024 * 1024
    }
}

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
