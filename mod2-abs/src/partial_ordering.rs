/*!

We need a version of `std::cmp::Ordering` that also has an `Unknown` or `Undecided` variant. In the Maude source, this 
enum is called `ReturnValue`.

There are also a couple of convenience free functions for converting a number to `Ordering` or `PartialOrdering`  based
on the sign of the number.

*/
use std::cmp::Ordering;

#[allow(non_snake_case)]
pub mod PartialOrdering {
  #![allow(non_upper_case_globals)]

  use std::cmp::Ordering;

  pub const Greater     : Option<Ordering> = Some(Ordering::Greater);
  pub const Less        : Option<Ordering> = Some(Ordering::Less);
  pub const Equal       : Option<Ordering> = Some(Ordering::Equal);
  pub const Incomparable: Option<Ordering> = None;
  pub const Unknown     : Option<Ordering> = None;
  
  
  pub fn from_sign<T>(value: T) -> Option<Ordering>
      where T: Into<isize>
  {
    let value = value.into();
    if value > 0 {
      Greater
    } else if value < 0 {
      Less
    } else {
      Equal
    }
  }
  
  pub fn from_ordering(value: Ordering) -> Option<Ordering> {
    match value {
      Ordering::Less    => Less,
      Ordering::Equal   => Equal,
      Ordering::Greater => Greater,
    }
  }
  
  #[inline(always)]
  pub fn from(ordering: Ordering) -> Option<Ordering> {
    from_ordering(ordering)
  }

}


#[inline(always)]
pub fn ordering_from_sign<T>(value: T) -> Ordering 
    where T: Into<isize> 
{
  let value: isize = value.into();
  
  if value > 0 {
    Ordering::Greater
  } else if value < 0 {
    Ordering::Less
  } else {
    Ordering::Equal
  }
}

