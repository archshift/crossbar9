use core::alloc::{GlobalAlloc, Layout};
use core::cell::Cell;
use core::ptr::{NonNull, null_mut};
use core::mem::{align_of, size_of};
use core::slice::{from_raw_parts, from_raw_parts_mut};
use core::ops::{Deref, DerefMut};

pub(crate) use ALLOCATOR as Global;

extern {
    static kernel_phys_end: usize;
}
fn main_mem_start() -> u32 {
    unsafe { &kernel_phys_end as *const usize as u32 }
}
fn main_mem_size() -> usize {
    (0x28000000 - main_mem_start()) as usize
}

pub struct Allocator {
    offset: Cell<usize>
}

impl Allocator {
    pub const fn new() -> Self {
        Self {
            offset: Cell::new(0)
        }
    }

    pub unsafe fn alloc_array<T>(&self, n: usize, align: usize) -> Result<NonNull<T>, ()> {
        let layout = Layout::from_size_align_unchecked(size_of::<T>() * n, align);
        let res = self.alloc(layout);
        if res == null_mut() {
            Err(())
        } else {
            Ok(NonNull::new_unchecked(res as *mut T))
        }
    }

    pub unsafe fn dealloc_array<T>(&self, ptr: NonNull<T>, n: usize, align: usize) {
        let layout = Layout::from_size_align_unchecked(size_of::<T>() * n, align);
        self.dealloc(ptr.as_ptr() as *mut u8, layout);
    }
}

unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let mut offset = self.offset.get();
        let padding = layout.align() - offset & (layout.align() - 1);
        offset += padding;
        let addr = main_mem_start() + offset as u32;
        offset += layout.size();

        self.offset.set(offset);
        if offset > main_mem_size() {
            null_mut()
        } else {
            addr as *mut u8
        }
    }
    
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

unsafe impl Sync for Allocator {}

pub struct Array<T> {
    ptr: NonNull<T>,
    size: usize,
    alignment: usize,
}

impl<T> Array<T> {
    pub fn new(size: usize) -> Self {
        Self::aligned_new(size, align_of::<T>())
    }

    pub fn aligned_new(size: usize, alignment: usize) -> Self {
        Self {
            ptr: unsafe { Global.alloc_array(size, alignment) }
                    .expect("Out of memory attempting to allocate array!"),
            size: size,
            alignment: alignment
        }
    }
}

impl<T> Deref for Array<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe { from_raw_parts(self.ptr.as_ptr(), self.size) }
    }
}

impl<T> DerefMut for Array<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe { from_raw_parts_mut(self.ptr.as_ptr(), self.size) }
    }
}

impl<T> Drop for Array<T> {
    fn drop(&mut self) {
        unsafe { Global.dealloc_array(self.ptr, self.size, self.alignment) };
    }
}
