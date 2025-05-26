/*!

Roughly speaking, an `OpDeclaration` corresponds to an op declaration in a source file:

```maude
op _+_ : Nat Nat -> Nat [assoc comm] . 
```

However, most of the attribute information is encoded elsewhere (in the symbol) and so does not appear in the 
`OpDeclaration` struct. The only thing we need to keep track of is whether the op is a constructor.

Recall: 

> Assuming that the equations in a functional module are (ground) Church-Rosser and terminating,
> then every ground term in the module (that is, every term without variables) will be simplified
> to a canonical form, perhaps modulo some declared equational attributes. Constructors are the
> operators appearing in such canonical forms. The operators that “disappear” after equational
> simplification are instead called defined functions. For example, typical constructors in a
> sort Nat are zero and s_, whereas in the sort Bool, true and false are the only constructors.

*/

use std::ops::{BitOr, BitOrAssign, Index, IndexMut};
use mod2_abs::SmallVec;
use crate::core::sort::SortPtr;


pub type TypeSignature = SmallVec<SortPtr, 1>;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
#[repr(u8)]
pub enum ConstructorStatus {
  // ToDo: Are the numeric values necessary?
  #[default]
  Unspecified    = 0,
  Constructor    = 1,
  NonConstructor = 2,
  Complex        = 1 | 2,
}

impl BitOr for ConstructorStatus {
  type Output = ConstructorStatus;

  #[inline(always)]
  fn bitor(self, rhs: Self) -> Self::Output {
    unsafe { std::mem::transmute(self as u8 | rhs as u8) }
  }
}

impl BitOrAssign for ConstructorStatus {
  #[inline(always)]
  fn bitor_assign(&mut self, rhs: Self) {
    unsafe { *self = std::mem::transmute(*self as u8 | rhs as u8) }
  }
}

impl From<bool> for ConstructorStatus {
  fn from(value: bool) -> Self {
    match value {
      true  => ConstructorStatus::Constructor,
      false => ConstructorStatus::NonConstructor
    }
  }
}


#[derive(PartialEq, Eq, Default)]
pub struct OpDeclaration {
  pub sort_spec     : TypeSignature,
  pub is_constructor: ConstructorStatus,
}

impl OpDeclaration {
  #[inline(always)]
  pub fn new(sort_spec: TypeSignature, is_constructor: ConstructorStatus) -> OpDeclaration {
    OpDeclaration { sort_spec, is_constructor }
  }

  #[inline(always)]
  pub fn len(&self) -> usize {
    self.sort_spec.len()
  }

  #[inline(always)]
  pub fn push(&mut self, sort: SortPtr) {
    self.sort_spec.push(sort);
  }
  
  #[inline(always)]
  pub fn iter(&self) -> core::slice::Iter<SortPtr> {
    self.sort_spec.iter()
  }

  #[inline(always)]
  pub fn arity(&self) -> i16 {
    (self.sort_spec.len() - 1) as i16
  }
}

impl Index<usize> for OpDeclaration {
  type Output = SortPtr;

  fn index(&self, index: usize) -> &Self::Output {
    self.sort_spec.index(index)
  }
}

impl IndexMut<usize> for OpDeclaration {
  fn index_mut(&mut self, index: usize) -> &mut Self::Output {
    self.sort_spec.index_mut(index)
  }
}