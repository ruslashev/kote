// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::mem;

use crate::arch::mmu::PAGE_SIZE;
use crate::mm::pg_alloc;

pub struct SmallVec<T> {
    buf: *mut T,
    len: usize,
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
        assert!(mem::size_of::<T>() != 0);
        assert!(mem::size_of::<U>() != 0);

        let ptr = slice.as_mut_ptr();
        let offset = ptr.align_offset(mem::align_of::<T>());

        assert!(offset < slice.len());

        let aligned = unsafe { ptr.add(offset) };

        Self {
            buf: aligned.cast::<T>(),
            len: 0,
            cap: (slice.len() * mem::size_of::<U>() - offset) / mem::size_of::<T>(),
            head: 0,
            tail: 0,
            view: 0,
        }
    }

    pub fn push_back(&mut self, item: T) {
        assert!(self.len < self.cap, "small_vec: overflow");

        unsafe {
            self.buf.add(self.tail).write(item);
        }

        self.len += 1;

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
        RoundRobinIterator {
            idx: self.view,
            len: self.len,
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

#[cfg(test)]
mod tests {
    use crate::small_vec::SmallVec;

    #[test]
    fn simple() {
        let mut storage = [0u8; 128];
        let mut vec = SmallVec::from_slice(&mut storage);

        vec.push_back(1u64);
        vec.push_back(2);
        vec.push_back(3);
        vec.push_back(4);
        vec.push_back(5);

        let mut it = vec.iter_round_robin();

        assert_eq!(it.next(), Some((0, &1)));
        assert_eq!(it.next(), Some((1, &2)));
        assert_eq!(it.next(), Some((2, &3)));
        assert_eq!(it.next(), Some((3, &4)));
        assert_eq!(it.next(), Some((4, &5)));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn full() {
        let mut storage = [0u64; 3];
        let mut vec = SmallVec::from_slice(&mut storage);

        vec.push_back(1u64);
        vec.push_back(2);
        vec.push_back(3);
    }

    #[test]
    #[should_panic(expected = "small_vec: overflow")]
    fn overflow() {
        let mut storage = [0u8; 30];
        let mut vec = SmallVec::from_slice(&mut storage);

        vec.push_back(1u64);
        vec.push_back(2);
        vec.push_back(3);
        vec.push_back(4);
    }

    #[test]
    fn one_element() {
        let mut storage = [0u64; 1];
        let mut vec = SmallVec::from_slice(&mut storage);

        vec.push_back(0u64);
    }

    #[test]
    #[should_panic(expected = "small_vec: overflow")]
    fn one_element_overflow() {
        let mut storage = [0u64; 1];
        let mut vec = SmallVec::from_slice(&mut storage);

        vec.push_back(0u64);
        vec.push_back(1u64);
    }

    #[test]
    #[should_panic]
    fn empty_push() {
        let mut storage = [0u8; 0];
        let mut vec = SmallVec::from_slice(&mut storage);

        vec.push_back(0u64);
    }

    #[test]
    fn empty_current() {
        let mut storage = [0u64; 4];
        let vec: SmallVec<u64> = SmallVec::from_slice(&mut storage);

        assert_eq!(vec.current(), None);
    }

    #[test]
    fn current() {
        let mut storage = [0u8; 128];
        let mut vec = SmallVec::from_slice(&mut storage);

        vec.push_back(0u64);
        vec.push_back(10);
        vec.push_back(20);
        vec.push_back(30);
        vec.push_back(40);

        vec.set_current(2);

        assert_eq!(vec.current(), Some(&mut 20));

        let mut it = vec.iter_round_robin();

        assert_eq!(it.next(), Some((2, &20)));
        assert_eq!(it.next(), Some((3, &30)));
        assert_eq!(it.next(), Some((4, &40)));
        assert_eq!(it.next(), Some((0, &0)));
        assert_eq!(it.next(), Some((1, &10)));
        assert_eq!(it.next(), None);
    }
}
