# Implementation Decisions

## Nullable indexes

The idiomatic way of representing a value that can be present or not present is, of course, with `Option<T>`. This comes
up a lot for different index types in Maude, and Maude implements these as `int`s, with nonnegative values as an index
and `-1` as `None` (and sometimes other special values as negative numbers). Maude's solution has some trade-offs:

| Pros                            | Cons                                                     |
| :------------------------------ | :------------------------------------------------------- |
| Simple                          | Can only represent half the possible unsigned values     |
| "conversion" to index is a noop | Special values are just convention, no semantics         |
| Same size as index type         | Type system doesn't enforce... anything                  |
|                                 | No mechanism prevents trying to index with special value |

But `Option<u32>` also has trade-offs:

| Pros                                 | Cons                                    |
| :----------------------------------- | :-------------------------------------- |
| Idiomatic                            | 8 bytes / 64 bits, twice the size!      |
| Special value has semantics          | Have to call `unwrap` for known indexes |
| Mechanism forces `None`/`Some` check | Mechanism forces `None`/`Some` check    |
|                                      | Conversion to index is not a noop       |

For applications in which indexes are a fundamental type used in bulk and in hot paths / tight loops, `Option<u32>`
isn't very attractive. An alternative is to do something like:

```rust
pub struct OptInt(NonZero<u32>);

impl OptInt {
  pub const ZERO: OptInt = OptInt(unsafe{ NonZero::new_unchecked(u32::ONE) });

  pub fn new(value: u32) -> Result<OptInt, ()> {
    value.checked_add(&u32::ONE)
         .map(|v| unsafe { OptInt(NonZero::new_unchecked(v)) })
         .ok_or(())
  }

  pub fn get(self) -> u32 {
    // Since `stored = original + 1 > 0`, this cannot underflow
    self.0.get() - u32::ONE
  }

  pub fn is_zero(self) -> bool {
    self.0.get().is_one()
  }
}
```

The represented value `v` is stored internally as `v + 1`. Since it wraps `NonZero<u32>`, `Option<OptInt>` is the same
size as `u32` and is "nullable."

| Pros                                      | Cons                                                    |
| :---------------------------------------- | :------------------------------------------------------ |
| Same size as `u32`                        | Conversion to `u32` is not a noop                       |
| Idiomatic                                 | Have to call `unwrap` AND `get` to extract an index     |
| Mechanism forces `None`/`Some` check      | Mechanism forces `None`/`Some` check                    |
| Generalizes to `n` special values         | Conversion from `u32` to `Option<OptInt>` needs a check |
| Only costs one `u32` value representation | Cannot represent `u32::MAX`                             |

The efficient use of space is attractive, but the programmer ergonomics aren't great, and doing arithmetic on every
single index operation is not very attractive for such a common operation.

We can combine a few of these to hopefully get the most attractive features in a single type. The idea is, instead of
using the all zero bit pattern to represent `None`, we use `u32::MAX`. In general, for an enum with `n` variants
representing "special" values, we use `u32::MAX`, `u32::MAX - 1`, `u32::MAX - n + 1` to represent these special values.
(Just subtract the discriminant from `u32::MAX`.)

```rust
pub struct SpecialIndex(u32);

impl SpecialIndex {
  const None: SpecialIndex = SpecialIndex(u32::MAX);
  //...
}
```

| Pros                                      | Cons                                                                             |
| :---------------------------------------- | :------------------------------------------------------------------------------- |
| Same size as `u32`                        | Not idiomatic                                                                    |
| Mechanism forces `None`/`Some` check      | Mechanism forces `None`/`Some` check                                             |
| Only costs one `u32` value representation | Cannot represent `u32::MAX`                                                      |
| Unsafe conversion to `u32` is a noop      | Safe conversion to `u32` requires `unwrap` and compiles to a check               |
| Generalizes to `n` special values         | Needs `EnumCount` or const param to generalize to `n` special values generically |

The ergonomics aren't great, but they aren't worse than `Option<u32>`, and when the value is known to be an index,
unchecked unwrapping is a noop.

I'm playing a little fast and loose with the words safe and unsafe. We can always convert from a `SpecialIndex` to a
`u32` (or `usize`) with safe code. The danger is that if we convert a `SpecialIndex::None` to a `u32`, we get a
`u32::MAX`, which is not what `SpecialIndex::None` represents and not what is likely intended. The API, therefore, needs
to enforce correct semantics. In practice, however, using the `SpecialIndex` as an index is so common that we want a
fast unchecked conversion to `usize`. We sacrifice some safety guards for ergonomics and pepper our code with
`debug_assert`s for some confidence that we do checks when they are needed.
