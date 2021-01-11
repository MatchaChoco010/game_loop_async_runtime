use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

pub struct Read<T: ?Sized + 'static> {
    value: &'static T,
}
impl<T: ?Sized + 'static> Deref for Read<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}
impl<T: ?Sized + 'static> Copy for Read<T> {}
impl<T: ?Sized + 'static> Clone for Read<T> {
    fn clone(&self) -> Self {
        Self { value: self.value }
    }
}

pub struct Write<T: ?Sized + 'static> {
    value: &'static mut T,
}
impl<T: ?Sized + 'static> Deref for Write<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}
impl<T: ?Sized + 'static> DerefMut for Write<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.value
    }
}

pub struct Container<T: ?Sized> {
    data: UnsafeCell<T>,
}
impl<T: Sized> Container<T> {
    pub fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }
}
impl<T: ?Sized> Container<T> {
    pub unsafe fn read(&self) -> Read<T> {
        Read {
            value: &*self.data.get(),
        }
    }

    pub unsafe fn write(&self) -> Write<T> {
        Write {
            value: &mut *self.data.get(),
        }
    }
}
unsafe impl<T: Sized> Sync for Container<T> {}
