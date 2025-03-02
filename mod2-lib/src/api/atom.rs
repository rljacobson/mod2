/*!

An `Atom` is an indivisible (atomic) element. An `Atom` can be
  - **Variable**: A pattern (or "meta") variable which may or may not be bound to a term, term sequence, or "null".
    - Singleton wildcard, also called a blank.
    - One-or-more wildcard, also called a blank sequence.
    - Zero-or-more wildcard, also called a null sequence.
  - **Data**: Atomic piece of data. For example, integer, string, byte, etc.
  - **Symbol**: Name which may or may not be bound to a term.

The type system is independent of these categories; that is, sorts/kinds are not represented by any of these entities.

# Defining Data Atoms

The `DataAtom` trait can be implemented for any type that implements `Display + Any + Eq + Hash`.

*/

use std::{
  any::Any,
  fmt::{
    Debug,
    Display,
    Formatter
  },
  hash::{Hash, Hasher}
};

pub use paste::paste;
use mod2_abs::DynHash;

use crate::{
  api::{
    symbol::{
      Symbol,
      SymbolPtr
    },
    variable::Variable
  }
};


#[derive(Eq, PartialEq, Hash)]
pub enum Atom {
  Variable(Variable),
  Symbol(SymbolPtr),
  Data(Box<dyn DataAtom>),
  // ToDo: Consider a built-in list type for "packed" data arrays
}

impl Atom {
  pub fn symbol(&self) -> SymbolPtr {
    match self {
      Atom::Variable(v) => v.symbol,
      Atom::Symbol(symbol) => *symbol,
      Atom::Data(data) => data.symbol()
    }
  }
}

impl Display for Atom {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {

      Atom::Variable(v) => {
        write!(f, "{}", v)
      }

      Atom::Symbol(symbol) => {
        unsafe {
          let symbol: &Symbol = &**symbol;
          write!(f, "{}", symbol)
        }
      }

      Atom::Data(data_atom) => {
        write!(f, "{}", data_atom)
      }

    }
  }
}

impl Debug for Atom {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    Display::fmt(self, f)
  }
}


/// The `DataAtomType` trait represents atomic pieces of data, like integers.
pub trait DataAtom: Display {
  // Implementers will implement the following verbatim.
  // fn as_any(&self) -> &dyn Any {
  //   self
  // }
  fn as_any(&self) -> &dyn Any;

  /// Equality between atoms of this kind
  fn eq(&self, other: &dyn DataAtom) -> bool;

  // / Forward hasher to data
  // fn hash(&self, state: &mut dyn Hasher);

  /// The symbol associated to this data type
  fn symbol(&self) -> SymbolPtr;
}

impl PartialEq for Box<dyn DataAtom> {
  fn eq(&self, other: &Self) -> bool {
    DataAtom::eq(&**self, &**other)
  }
}

impl Eq for Box<dyn DataAtom> {}

// impl Hash for Box<dyn DataAtom> {
//   fn hash<H: Hasher>(&self, state: &mut H) {
//     DataAtom::hash(&**self, state)
//   }
// }

impl Hash for dyn DataAtom {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.dyn_hash(state)
  }
}

/**
# `implement_data_atom!` Macro

This macro generates a newtype around an existing type, with a name that is derived from a provided identifier and suffixed with "Atom". It also implements several standard traits (`PartialEq`, `Eq`, `Debug`, `Hash`, and `Display`) as well as the `DataAtom` trait for the newly created type. The `DataAtom` implementation includes a unique symbol associated with the new type, which is constructed using the provided name in string form.

## Parameters

- `$name`: The base name of the new type. This name will be suffixed with `Atom` to create the new type. For example, if you pass `Integer`, the new type will be `IntegerAtom`.

- `$type`: The underlying type for the newtype. This defines the type of data that the new `Atom` struct will hold. It must implement `Display + Any + PartialEq + Eq + Hash`.

## Generated Code

When called, the macro expands into the following:

1. **Newtype Definition**: A new struct is created with the name `$nameAtom`, wrapping the existing type `$type`.

2. **A Static Symbol**: A lazily allocated static symbol `$NAME_SYMBOL` (where `$name` is converted to screaming snake case). A pointer to this symbol can be acquired through the member function `$nameAtom::symbol()`. For example, `implement_data_atom!(Integer)` defines `INTEGER_SYMBOL`.

2. **Trait Implementations**:
   - `PartialEq`, `Eq`, `Debug`, `Hash`: These standard traits are automatically derived for the newtype.
   - `Display`: Implements the `Display` trait to output the inner value of the newtype, using the `Display` trait of the inner type.
   - `DataAtom`: Implements a custom `DataAtom` trait, where the name provided in the macro call is used to construct a static symbol. The name is included in the `DataAtom` implementation via the `symbol()` method, which returns a cached `SymbolPtr` that contains metadata about the type (name, arity, etc.).

## Example Usage

```rust
use std::any::Any;
use once_cell::sync::Lazy;
use mod2_lib::api::{
  Arity,
  atom::{
    implement_data_atom, Atom, DataAtom
  },
  symbol::{Symbol, SymbolPtr, SymbolType, SymbolAttribute}
};
use mod2_abs::IString;

implement_data_atom!(Integer, isize);

fn main() {
    let int_atom = IntegerAtom::new_atom(42isize);

    // The Display implementation allows it to be printed
    println!("The data atom is {}.", int_atom);
    // Access the associated symbol for the new type. Use `unsafe` to dereference the pointer.
    let int_symbol: &Symbol = unsafe { &*int_atom.symbol() };
    println!("Its symbol is {}.", int_symbol);
}
```

*/

#[macro_export]
macro_rules! implement_data_atom {
  ($name:ident, $type:ty) => {
    $crate::api::atom::paste!{

    // Define the newtype with the name appended with "Atom"
    #[derive(PartialEq, Eq, Debug, Hash)]
    pub struct [<$name Atom>]($type);

    impl [<$name Atom>] {
      /// Creates a new `Atom::Data` containing a boxed `DataAtom` wrapping `data`
      pub fn new_atom(data: $type) -> Atom {
        Atom::Data(Box::new([<$name Atom>](data)))
      }
    }

    impl std::fmt::Display for [<$name Atom>] {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
      }
    }

    impl DataAtom for [<$name Atom>] {

      fn as_any(&self) -> &dyn Any {
        self
      }

      fn eq(&self, other: &dyn DataAtom) -> bool {
        if let Some(other) = other.as_any().downcast_ref::<[<$name Atom>]>() {
          self.0 == other.0
        } else {
          false
        }
      }

      fn symbol(&self) -> SymbolPtr {
        let ptr: *const Symbol = unsafe{&*[<$name:snake:upper _SYMBOL>]};
        ptr as SymbolPtr
      }
    }

    #[allow(non_upper_case_globals)]
    pub static [<$name:snake:upper _SYMBOL>]: Lazy<Symbol> = Lazy::new(|| {
      Symbol {
          name:        IString::from(stringify!($name)),  // Use the identifier as a string
          // ToDo: What should the arity of a `DataAtom` have?
          arity:       Arity::Unspecified,
          attributes:  SymbolAttribute::Constructor.into(),
          symbol_type: SymbolType::Data,
          hash_value:  0
        }
    });

    } // end paste!
  }; // end macro pattern
}
pub use implement_data_atom;


