use std::cell::UnsafeCell;

pub struct UnsafeMutSlice<'a, T> {
    slice: UnsafeCell<&'a mut [T]>,
}
impl<'a, T> UnsafeMutSlice<'a, T> {
    /// # Safety
    /// The caller must ensure that no two threads modify the same index at the same time.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get(&self, index: usize) -> &mut T {
        let slice = unsafe { &mut *self.slice.get() };
        &mut slice[index]
    }

    pub fn new(slice: &'a mut [T]) -> Self {
        Self {
            slice: UnsafeCell::new(slice),
        }
    }
}

unsafe impl<'a, T> Sync for UnsafeMutSlice<'a, T> {}

pub struct UnsafeMut<T> {
    item: UnsafeCell<T>,
}

impl<T> UnsafeMut<T> {
    /// # Safety
    /// The caller must ensure that no two threads modify the same part of the item at the same time.
    #[allow(clippy::mut_from_ref)]
    pub unsafe fn get(&self) -> &mut T {
        let item = unsafe { &mut *self.item.get() };
        &mut *item
    }

    pub fn new(item: T) -> Self {
        Self {
            item: UnsafeCell::new(item),
        }
    }
}

unsafe impl<T> Sync for UnsafeMut<T> {}
