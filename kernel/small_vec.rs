// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::ops::{Index, IndexMut};
use core::{mem, slice};

use crate::arch::mmu::PAGE_SIZE;
use crate::mm::pg_alloc;

pub struct SmallVec<T> {
    storage: *mut T,
    size: usize,
    capacity: usize,
}

impl<T> SmallVec<T> {
    pub fn new() -> Self {
        let page = pg_alloc::alloc_page();
        let addr = page.to_physaddr().into_vaddr().0;

        Self {
            storage: addr as *mut T,
            size: 0,
            capacity: PAGE_SIZE / mem::size_of::<T>(),
        }
    }

    pub fn push(&mut self, item: T) {
        assert!(self.size < self.capacity);

        self.size += 1;

        unsafe {
            self.last_element().write(item);
        }
    }

    pub fn delete_at(&mut self, idx: usize) {
        assert!(self.size > 0 && idx < self.size);

        unsafe {
            let del_ptr = self.storage.add(idx);

            self.last_element().swap(del_ptr);

            drop(self.last_element().read());
        }

        self.size -= 1;
    }

    unsafe fn last_element(&mut self) -> *mut T {
        self.storage.add(self.size - 1)
    }
}

impl<T> Index<usize> for SmallVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        let slice = unsafe { slice::from_raw_parts(self.storage, self.size) };

        &slice[index]
    }
}

impl<T> IndexMut<usize> for SmallVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        let slice_mut = unsafe { slice::from_raw_parts_mut(self.storage, self.size) };

        &mut slice_mut[index]
    }
}

impl<T> Drop for SmallVec<T> {
    fn drop(&mut self) {
        for i in 0..self.size {
            let elem = unsafe { self.storage.add(i).read() };
            drop(elem);
        }
    }
}
