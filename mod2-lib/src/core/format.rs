/*!

There are different text representations possible for terms, DAGs, and so forth, that we want depending on the context.
This module provides a unified API for formatting objects across the project.

*/


use std::fmt::Display;

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
  fn repr(&self, style: FormatStyle) -> String;
}

/**
Any type that implements `Formattable` can trivially implement `Display` as in this example:

```ignore
impl Display for FreeTerm {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    (self as &dyn Term).fmt(f)
  }
}
```
*/
impl Display for dyn Formattable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", Formattable::repr(self, FormatStyle::Default))
  }
}

// impl Display for Box<dyn Formattable> {
//   fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
//     write!(f, "{}", &**self)
//   }
// }
