use std::iter::once;
use std::fmt::Display;

/**
Join an iterator of strings, which doesn't exist in the stdlib. (C.f. `Vec::join(…)`)

From: <https://stackoverflow.com/a/66951473>
Usage:

```
# use mod2_abs::join_iter;
let iter = [1, 3, 5, 7, 9].iter().cloned();
println!("{:?}", join_iter(iter, |v| v - 1).collect::<Vec<_>>());
// [1, 2, 3, 4, 5, 6, 7, 8, 9]

let iter = ["Hello", "World"].iter().cloned();
let sep = ", ";
println!("{:?}", join_iter(iter, |_| sep).collect::<String>());
// "Hello, World"
```
*/
pub fn join_iter<T>(mut iter: impl Iterator<Item = T>, sep: impl Fn(&T) -> T)
                    -> impl Iterator<Item = T>
{
  iter
      .next()
      .into_iter()
      .chain(iter.flat_map(move |s| once(sep(&s)).chain(once(s))))
}

/// Join a list of things that can be displayed as string with a given separator.
///
/// This is a convenience function that defers to `join_iter`.
pub fn join_string<T:Display>(iter: impl Iterator<Item = T>, sep: &str) -> String {
  join_iter(iter.map(|t| t.to_string()), |_| sep.to_string()).collect::<String>()
}

/// Expresses the number using Unicode subscript digits
pub fn int_to_subscript<T: Into<u32>>(value: T) -> String {
  let value: u32 = value.into();
  let subscript_digits = ['₀', '₁', '₂', '₃', '₄', '₅', '₆', '₇', '₈', '₉'];
  value
      .to_string()
      .chars()
      .map(|c| match c {
        '0'..='9' => subscript_digits[c as usize - '0' as usize],
        _ => c, // Handle unexpected characters, though there shouldn't be any
      })
      .collect()
}



#[cfg(test)]
mod tests {
  use super::{join_iter, join_string};

  #[test]
  fn join_iter_test() {
    let iter = [1, 3, 5, 7, 9].iter().cloned();
    let joined = format!("{:?}", join_iter(iter, |v| v - 1).collect::<Vec<_>>());
    assert_eq!(joined, "[1, 2, 3, 4, 5, 6, 7, 8, 9]");
    // [1, 2, 3, 4, 5, 6, 7, 8, 9]

    let iter = ["Hello", "World"].iter().cloned();
    let sep = ", ";
    let joined = format!("{:?}", join_iter(iter, |_| sep).collect::<String>());
    assert_eq!(joined, "\"Hello, World\"");
    // "Hello, World"
  }

  #[test]
  fn join_string_test(){
    let list = [1, 3, 5, 7, 9];
    let sep = ", ";
    let joined = join_string(list.iter(), sep);
    assert_eq!(joined, "1, 3, 5, 7, 9");
  }
}
