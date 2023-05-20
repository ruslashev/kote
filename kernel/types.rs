// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use crate::arch::mmu::{PAGE_SIZE, PAGE_SIZE_LARGE};
use crate::mm::types::{PhysAddr, VirtAddr};

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
    #[inline]
    fn is_po2_aligned(self, po2: Self) -> bool {
        self & (po2 - 1) == 0
    }
    #[inline]
    fn po2_round_down(self, po2: Self) -> Self {
        self & !(po2 - 1)
    }
    #[inline]
    fn po2_round_up(self, po2: Self) -> Self {
        (self + po2 - 1) & !(po2 - 1)
    }

    #[inline]
    fn is_page_aligned(self) -> bool {
        self.is_po2_aligned(PAGE_SIZE as Self)
    }
    #[inline]
    fn page_round_down(self) -> Self {
        self.po2_round_down(PAGE_SIZE as Self)
    }
    #[inline]
    fn page_round_up(self) -> Self {
        self.po2_round_up(PAGE_SIZE as Self)
    }

    #[inline]
    fn is_lpage_aligned(self) -> bool {
        self.is_po2_aligned(PAGE_SIZE_LARGE as Self)
    }
    #[inline]
    fn lpage_round_down(self) -> Self {
        self.po2_round_down(PAGE_SIZE_LARGE as Self)
    }
    #[inline]
    fn lpage_round_up(self) -> Self {
        self.po2_round_up(PAGE_SIZE_LARGE as Self)
    }
}
        )*
    }
}

impl_po2_ops!(u32 u64 usize);

macro_rules! impl_po2_ops_for_newtypes {
    ( $( $type:ident )+ ) => {
        $(
impl PowerOfTwoOps for $type {
    #[inline]
    fn is_po2_aligned(self, po2: Self) -> bool {
        self.0.is_po2_aligned(po2.0)
    }
    #[inline]
    fn po2_round_down(self, po2: Self) -> Self {
        Self(self.0.po2_round_down(po2.0))
    }
    #[inline]
    fn po2_round_up(self, po2: Self) -> Self {
        Self(self.0.po2_round_up(po2.0))
    }

    #[inline]
    fn is_page_aligned(self) -> bool {
        self.0.is_page_aligned()
    }
    #[inline]
    fn page_round_down(self) -> Self {
        Self(self.0.page_round_down())
    }
    #[inline]
    fn page_round_up(self) -> Self {
        Self(self.0.page_round_up())
    }

    #[inline]
    fn is_lpage_aligned(self) -> bool {
        self.0.is_lpage_aligned()
    }
    #[inline]
    fn lpage_round_down(self) -> Self {
        Self(self.0.lpage_round_down())
    }
    #[inline]
    fn lpage_round_up(self) -> Self {
        Self(self.0.lpage_round_up())
    }
}
        )*
    }
}

impl_po2_ops_for_newtypes!(VirtAddr PhysAddr);

#[const_trait]
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
