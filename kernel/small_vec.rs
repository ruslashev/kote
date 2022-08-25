// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::mem;

use crate::arch::mmu::PAGE_SIZE;
use crate::mm::pg_alloc;

pub struct SmallVec<T> {
    buf: *mut T,
    cap: usize,
    head: usize,
    tail: usize,
    view: usize,
}

impl<T> SmallVec<T> {
    pub fn new() -> Self {
        let page = pg_alloc::alloc_page();
        let addr = page.to_physaddr().into_vaddr();
        let slice = unsafe { addr.into_slice_mut(PAGE_SIZE) };

        Self::from_slice(slice)
    }

    pub fn from_slice<U>(slice: &mut [U]) -> Self {
        let ptr = slice.as_mut_ptr();
        let offset = ptr.align_offset(mem::align_of::<T>());

        assert!(offset < slice.len());

        let aligned = unsafe { ptr.add(offset) };

        Self {
            buf: aligned.cast::<T>(),
            cap: (slice.len() - offset) / mem::size_of::<T>(),
            head: 0,
            tail: 0,
            view: 0,
        }
    }

    pub fn push_back(&mut self, item: T) {
        assert!((self.tail + 1) % self.cap != self.head, "small_vec: overflow");

        unsafe {
            self.buf.add(self.tail).write(item);
        }

        self.tail += 1;
        self.tail %= self.cap;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.head == self.tail {
            return None;
        }

        let elem = unsafe { self.buf.add(self.head).read() };

        self.head += 1;
        self.head %= self.cap;

        if self.view < self.head {
            self.view = self.head;
        }

        Some(elem)
    }

    pub fn current(&self) -> Option<&mut T> {
        if self.head == self.tail {
            None
        } else {
            unsafe { self.buf.add(self.view).as_mut() }
        }
    }

    pub fn set_current(&mut self, new_view: usize) {
        assert!(new_view >= self.head && new_view < self.tail);

        self.view = new_view;
    }

    pub fn iter_round_robin(&self) -> RoundRobinIterator<T> {
        let len = if self.head < self.tail {
            self.tail - self.head
        } else {
            self.tail + self.cap - self.head
        };

        RoundRobinIterator {
            idx: self.view,
            len,
            vec: self,
        }
    }
}

impl<T> Drop for SmallVec<T> {
    fn drop(&mut self) {
        while let Some(elem) = self.pop_front() {
            drop(elem);
        }
    }
}

pub struct RoundRobinIterator<'a, T> {
    idx: usize,
    len: usize,
    vec: &'a SmallVec<T>,
}

impl<'a, T> Iterator for RoundRobinIterator<'a, T> {
    type Item = (usize, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            let idx = self.idx;
            let elem = unsafe { self.vec.buf.add(idx).as_ref().unwrap() };

            self.len -= 1;

            self.idx += 1;
            if self.idx == self.vec.tail {
                self.idx = self.vec.head;
            }

            Some((idx, elem))
        }
    }
}
