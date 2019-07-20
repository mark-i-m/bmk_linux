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
    ptr: *mut u8,
    size: usize, // in bytes

    cap: usize, // size as number of elements
    len: usize, // number of pushed elements

    _phantom: core::marker::PhantomData<T>,
}

impl<T: Sized> ResultArray<T> {
    /// Create a new `ResultArray` with the given number of elements.
    ///
    /// # Panics
    ///
    /// - If unable to create the array.
    /// - If the size of the array is not a multiple of the page size.
    /// - If the length of the array is longer than `isize::MAX`.
    pub fn new(cap: usize) -> Self {
        let size = cap * mem::size_of::<T>();

        assert!(size % PAGE_SIZE == 0);
        assert!(size < (std::isize::MAX as usize));

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
            ptr: mapped,
            size,
            len: 0,
            cap,
            _phantom: core::marker::PhantomData,
        }
    }

    /// Returns an iterator over only pushed elements.
    pub fn iter(&self) -> slice::Iter<T> {
        self.as_slice().iter()
    }

    /// Push the `item` to the end of the array.
    ///
    /// # Panics
    ///
    /// if the array is full.
    pub fn push(&mut self, item: T) {
        assert!(self.len < self.cap); // full?

        let ptr = unsafe { (self.ptr as *mut T).offset(self.len as isize) };
        self.len += 1;
        unsafe {
            std::ptr::write(ptr, item);
        }
    }

    fn as_slice(&self) -> &[T] {
        unsafe { core::slice::from_raw_parts(self.ptr as *const T, self.len) }
    }

    /// Pop from the end and drop the popped value.
    ///
    /// # Panics
    ///
    /// if the array is empty.
    fn pop(&mut self) {
        assert!(self.len > 0);

        self.len -= 1;
        let ptr = unsafe { self.ptr.offset(self.len as isize) };
        unsafe {
            let _ = std::ptr::read(ptr as *const T);
        }
    }
}

impl<T: Sized> Drop for ResultArray<T> {
    fn drop(&mut self) {
        // Drop all values
        while self.len > 0 {
            self.pop();
        }

        // Unmap memory
        unsafe {
            libc_munmap(self.ptr as *mut _, self.size);
        }
    }
}
