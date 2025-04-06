/*!

Generic memory utilities.

*/

#[inline(always)]
pub fn as_bytes<T: Sized>(value: &T) -> &[u8] {
  unsafe {
    std::slice::from_raw_parts(value as *const T as *const u8, std::mem::size_of::<T>())
  }
}

#[inline(always)]
pub fn as_bytes_mut<T: Sized>(value: &mut T) -> &mut [u8] {
  unsafe {
    std::slice::from_raw_parts_mut(value as *mut T as *mut u8, std::mem::size_of::<T>())
  }
}

