//! A safe mlocked array in memory.

use std::{mem, ptr, slice};

use libc::{
    mlock as libc_mlock, mmap as libc_mmap, munmap as libc_munmap, MAP_ANONYMOUS, MAP_FAILED,
    MAP_PRIVATE, PROT_READ, PROT_WRITE,
};

/// Number of bytes in a page.
pub const PAGE_SIZE: usize = 1 << 12; // 4KB

/// A pre-allocated, mlocked, and prefaulted array of the given size and type for storing results.
/// This is useful to the storage of results from interfering with measurements.
pub struct ResultArray<T: Sized> {
    array: Option<Vec<T>>,
}

impl<T: Sized> ResultArray<T> {
    /// Create a new `ResultArray` with the given number of elements.
    ///
    /// # Panics
    ///
    /// - If unable to create the array.
    /// - If the size of the array is not a multiple of the page size.
    pub fn new(nelem: usize) -> Self {
        let size = nelem * mem::size_of::<T>();

        assert!(size % PAGE_SIZE == 0);

        // Get the virtual address space.
        let mapped = unsafe {
            let addr = libc_mmap(
                ptr::null_mut(),
                size,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANONYMOUS,
                -1,
                0,
            );

            if addr == MAP_FAILED {
                panic!("Unable to mmap");
            }

            addr as *mut _
        };

        // Populate and lock the whole array
        unsafe {
            let ret = libc_mlock(mapped as *const _, size);
            assert_eq!(ret, 0);
        }

        Self {
            array: unsafe { Some(Vec::from_raw_parts(mapped, 0, nelem)) },
        }
    }

    pub fn iter(&self) -> slice::Iter<T> {
        self.array.as_ref().unwrap().iter()
    }

    pub fn push(&mut self, item: T) {
        self.array.as_mut().unwrap().push(item);
    }
}

impl<T: Sized> Drop for ResultArray<T> {
    fn drop(&mut self) {
        // Drain the vec
        drop(self.array.as_mut().unwrap().drain(0..));

        // munmap
        let mut array = self.array.take().unwrap();
        let size = array.capacity() * mem::size_of::<T>();
        let ptr = array.as_mut_ptr();

        mem::forget(array); // never call `Vec::drop`

        unsafe {
            libc_munmap(ptr as *mut _, size);
        }
    }
}
