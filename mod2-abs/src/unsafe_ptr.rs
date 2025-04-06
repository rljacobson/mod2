/*!

An `UnsafePtr<T>` is a smart pointer type that subverts Rust's borrowing and safety rules. It wraps a
raw pointer to memory that it does not manage. This type is fundamentally unsafe and should only be
used in circumstances where the programmer can guarantee that the memory pointed to by the raw
pointer is valid for the lifetime of the `UnsafePtr<T>`.

Equality semantics of `UnsafePtr<T>` are semantics of fat pointer equality: Two `UnsafePtr<T>`s are
equal if they point to the same address *and* have the same vtable pointer. Rust pointers are in general
fat pointers, which consist of a pointer to a vtable and an address pointer. An `UnsafePtr<dyn Symbol>`
and an `UnsafePtr<dyn Formattable>` can point to the same address but have different bit patterns,
because they have different vtables.

For address equality semantics, use `UnsafePtr<T>::add_eq`.

*/

use std::{
  fmt::{Display, Formatter, Debug},
  ops::{Deref, DerefMut},
  ptr::NonNull
};
use std::hash::{Hash, Hasher};

pub struct UnsafePtr<T: ?Sized> {
  ptr: NonNull<T>,
}

// For some reason the compiler can't tell the type is `PartialEq` and `Eq` when they are derived.
/// Equality semantics of `UnsafePtr<T>` are semantics of fat pointer equality.
impl<T: ?Sized> PartialEq for UnsafePtr<T> {
  fn eq(&self, other: &Self) -> bool {
    std::ptr::eq(self.ptr.as_ptr(),  other.ptr.as_ptr())
  }
}
impl<T: ?Sized> Eq for UnsafePtr<T> {}

// For some reason the compiler can't tell the type is `Copy` and `Clone` when they are derived.
impl<T: ?Sized> Clone for UnsafePtr<T> {
  fn clone(&self) -> Self {
    Self{
      ptr: self.ptr,
    }
  }
}
impl<T: ?Sized> Copy for UnsafePtr<T> {}

// ToDo: Audit usages for thread safety.
unsafe impl<T: ?Sized> Sync for UnsafePtr<T> {}
unsafe impl<T: ?Sized> Send for UnsafePtr<T> {}

impl<T: ?Sized> UnsafePtr<T> {
  pub fn new(ptr: *mut T) -> Self {
    assert!(!ptr.is_null());
    Self { ptr: unsafe{ NonNull::new_unchecked(ptr) } }
  }

  pub fn as_ptr(&self) -> *const T {
    self.ptr.as_ptr()
  }

  pub fn as_mut_ptr(&self) -> *mut T {
    self.ptr.as_ptr()
  }

  pub fn addr_eq(&self, rhs: UnsafePtr<T>) -> bool {
    std::ptr::addr_eq(self.ptr.as_ptr(), rhs.ptr.as_ptr())
  }

  /// If `self` and `rhs` are both pointers to the same dyn trait type, this method can tell you if they have the same
  /// concrete type. Warning: This is not reliable for determining if the concrete types are different. 
  pub fn vtable_eq(&self, rhs: UnsafePtr<T>) -> bool {
    std::ptr::metadata(self.ptr.as_ptr()) == std::ptr::metadata(rhs.ptr.as_ptr())
  }
}

impl<T: ?Sized> Deref for UnsafePtr<T> {
  type Target = T;

  fn deref(&self) -> &Self::Target {
    unsafe { self.ptr.as_ref() }
  }
}

impl<T: ?Sized> DerefMut for UnsafePtr<T> {
  fn deref_mut(&mut self) -> &mut Self::Target {
    unsafe { self.ptr.as_mut() }
  }
}

impl<T: Display + ?Sized> Display for UnsafePtr<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Display::fmt(unsafe { self.ptr.as_ref() }, f)
  }
}

impl<T: Debug + ?Sized> Debug for UnsafePtr<T> {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Debug::fmt(unsafe { self.ptr.as_ref() }, f)
  }
}

impl<T: ?Sized + Hash> Hash for UnsafePtr<T> {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.deref().hash(state);
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

    let maybe_x: Option<UnsafePtr<i32>> = None;

    println!("size of UnsafePtr<i32>: {}", size_of::<UnsafePtr<i32>>());
    println!("size of Option<UnsafePtr<i32>>: {}", size_of::<Option<UnsafePtr<i32>>>());

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
