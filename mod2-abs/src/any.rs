/*!

Macros to implement `as_any` and `as_ptr` (`UnsafePtr`).

Note that `as_ptr()` should only be used for pinned heap allocated objects.

To generate the definition:

```rust
# use mod2_abs::UnsafePtr;
type MyTraitPtr = UnsafePtr<dyn MyTrait>;

pub trait MyTrait{

  fn as_any(&self) -> &dyn std::any::Any;
  fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
  /// This is very unsafe. Make sure this object is heap allocated and pinned before calling.
  fn as_ptr(&self) -> MyTraitPtr;

  fn some_other_fn(&self) -> i32 {
    42
  }
}

struct MyStruct;

impl MyTrait for MyStruct {
  fn as_any(&self) -> &dyn std::any::Any { self }
  fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
  fn as_ptr(&self) -> MyTraitPtr {
    MyTraitPtr::new(self as *const dyn MyTrait as *mut dyn MyTrait)
  }

  fn some_other_fn(&self) -> i32 { 42 }
}
```

write the following:

```rust
# use mod2_abs::{decl_as_any_ptr_fns, impl_as_any_ptr_fns, UnsafePtr};
type MyTraitPtr = UnsafePtr<dyn MyTrait>;

pub trait MyTrait{

  decl_as_any_ptr_fns!(MyTrait);

  fn some_other_fn(&self) -> i32 {
    42
  }
}

struct MyStruct;

impl MyTrait for MyStruct {
  impl_as_any_ptr_fns!(MyTrait, MyStruct);
  
  fn some_other_fn(&self) -> i32 { 42 }
}
```




*/

use std::any::Any;
use crate::UnsafePtr;
pub use paste::paste;


#[macro_export]
macro_rules! decl_as_any_ptr_fns {
    ($trait_name:ident) => {
      fn as_any(&self)         -> &dyn std::any::Any;
      fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
      fn as_ptr(&self)         -> $crate::any::paste!{[<$trait_name Ptr>]};
    };
}
pub use decl_as_any_ptr_fns;

#[macro_export]
macro_rules! impl_as_any_ptr_fns {
    ($trait_name:ident, $struct_name:ident) => {
      // $crate::any::paste!{
      //   pub type [<$trait_name Ptr>]  = $crate::UnsafePtr<dyn $trait_name>;
      // }
      
      // It turns out you cannot split a trait impl over two blocks. 
      // impl $trait_name for $struct_name {
        fn as_any(&self)         -> &dyn std::any::Any { self }
        fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
        fn as_ptr(&self)         -> $crate::any::paste!{[<$trait_name Ptr>]} {
          $crate::any::paste!{
            [<$trait_name Ptr>]::new(self as *const dyn $trait_name as *mut dyn $trait_name) 
          }
        }
      // }
    };
}
pub use impl_as_any_ptr_fns;


