/*!

An `UnsafeRef` is a smart pointer type that subverts Rust's borrowing and safety rules. It wraps a
raw pointer to memory that it does not manage. This type is fundamentally unsafe and should only be
used in circumstances where the programmer can guarantee that the memory pointed to by the raw
pointer is valid for the lifetime of the `UnsafeRef`.

*/

use std::{
  fmt::{Display, Formatter, Debug},
  ops::{Deref, DerefMut}
};


#[derive(Eq, PartialEq)]
pub struct UnsafePtr<T: ?Sized> {
  ptr: *mut T,
}

impl<T: ?Sized> Clone for UnsafePtr<T> {
  fn clone(&self) -> Self {
    Self{
      ptr: self.ptr,
    }
  }
}
impl<T: ?Sized> Copy for UnsafePtr<T> {}
unsafe impl<T: ?Sized> Sync for UnsafePtr<T> {}
unsafe impl<T: ?Sized> Send for UnsafePtr<T> {}

impl<T: ?Sized> UnsafePtr<T> {
  pub fn new(ptr: *mut T) -> Self {
    assert!(!ptr.is_null());
    Self { ptr }
  }

  pub fn as_ptr(&self) -> *const T {
    self.ptr
  }

  pub fn as_mut_ptr(&self) -> *mut T {
    self.ptr
  }
  
  pub fn addr_eq<U>(&self, rhs: &Self) -> bool
      where U: Deref<Target=T>
  {
    std::ptr::addr_eq(self.ptr, rhs.ptr)
  }
}

impl<T: ?Sized> Deref for UnsafePtr<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { &*self.ptr }
  }
}

impl<T: ?Sized> DerefMut for UnsafePtr<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { &mut *self.ptr }
  }
}

impl<T: Display + ?Sized> Display for UnsafePtr<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Display::fmt(unsafe { &*self.ptr }, f)
  }
}

impl<T: Debug + ?Sized> Debug for UnsafePtr<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Debug::fmt(unsafe { &*self.ptr }, f)
  }
}


#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn test_unsafe_ref() {
    let mut x = 10;
    let mut y = 20;
    let mut z = 30;

    let mut x_ref = UnsafePtr::new(&mut x as *mut i32);
    let mut y_ref = UnsafePtr::new(&mut y as *mut i32);
    let mut z_ref = UnsafePtr::new(&mut z as *mut i32);

    assert_eq!(*x_ref, 10);
    assert_eq!(*y_ref, 20);
    assert_eq!(*z_ref, 30);

    println!("x_ref: {}", x_ref);
    println!("y_ref: {}", y_ref);
    println!("z_ref: {}\n", z_ref);

    *x_ref = 100;
    *y_ref = 200;
    *z_ref = 300;

    assert_eq!(*x_ref, 100);
    assert_eq!(*y_ref, 200);
    assert_eq!(*z_ref, 300);

    println!("x_ref: {}", x_ref);
    println!("y_ref: {}", y_ref);
    println!("z_ref: {}", z_ref);

  }
}
