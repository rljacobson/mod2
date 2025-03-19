/*!

There are different text representations possible for terms, DAGs, and so forth, that we want depending on the context.
This module provides a unified API for formatting objects across the project. 

The trait that types that can be formatted implement is `Formattable`. It works similar to the standard library's 
`Display` trait. Unfortunately, `Display` can't be extended with formatting for user defined types. Both `Debug` and 
`Display` are implemented for `dyn Formattable`, but this isn't enough to implement these traits for `T: Formattable`. 
We provide a convenience macro that does so:

```rust
# use mod2_lib::core::format::{Formattable, FormatStyle, impl_display_debug_for_formattable};
struct MyStruct;
impl Formattable for MyStruct {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) 
      -> std::fmt::Result
  {
    write!(f, "MyStruct<{}>", style)
  }
}
impl_display_debug_for_formattable!(MyStruct)
```

*/

use std::fmt::Debug;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum FormatStyle {
  #[default]
  Default, // Use the default formatting
  Simple, // Use a simplified formatting
  Input,  // Format the term as a valid input expression, if possible.
  Debug,  // Format with extra debugging information
}

pub trait Formattable {
  /// Writes a text representation of `self` according to the given `FormatStyle`.
  /// Use `format!` and friends to create a string.
  fn repr(&self, out: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result;
}

#[macro_export]
macro_rules! impl_display_debug_for_formattable {
    ($t:ty) => {
        impl std::fmt::Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Use the `repr` method with FormatStyle::Default
                <$t as $crate::core::format::Formattable>::repr(self, f, $crate::core::format::FormatStyle::Default)
            }
        }

        impl std::fmt::Debug for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                // Use the `repr` method with FormatStyle::Debug
                <$t as $crate::core::format::Formattable>::repr(self, f, $crate::core::format::FormatStyle::Debug)
            }
        }
    };
}
pub use impl_display_debug_for_formattable;