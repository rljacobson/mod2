/*!

Definitions related to symbols. Symbols can be thought of as names to which additional information is attached, such as
arity and theory axioms.

In an expression like, `f[x, Thing1, 45]`, the symbols are `f`, `x`, and `Thing1`. There is also an implicit symbol
shared by all data constants, the number `45` in this case, which is defined by the client code that defined the
`DataAtom` type. Integers might be represented by the `IntegerAtom` type (implementing the `DataAtom` trait) and have
the symbol `Integer` for example.

*/

use std::fmt::Display;

use enumflags2::{bitflags, make_bitflags, BitFlags};
use mod2_abs::{
  int_to_subscript,
  Set,
  IString
};

use crate::{
  api::Arity,
  core::format::{FormatStyle, Formattable},
  core::sort::sort_table::SortTable,
};
use crate::core::sort::Sort;
use crate::core::sort::sort_spec::SortSpec;

pub type SymbolPtr = *mut Symbol;
pub type SymbolSet = Set<Symbol>;


#[derive(Eq, PartialEq, Hash)]
pub struct Symbol {
  pub name       : IString,

  pub arity      : Arity,
  pub attributes : SymbolAttributes,
  pub symbol_type: SymbolType,
  
  sort_table: Option<SortTable>,

  /// Unique integer for comparing symbols, also called order. In Maude, the `order`
  /// has lower bits equal to the value of an integer that is incremented every time
  /// a symbol is created and upper 8 bits (bits 24..32) equal to the arity. Note:
  /// We enforce symbol creation with `Symbol::new()` by making hash_value private.
  hash_value : u32,
}

// This is an abomination. See `api/built_in/mod.rs`.
unsafe impl Send for Symbol {}
unsafe impl Sync for Symbol {}

impl Symbol {
  /// All symbols must be created with `Symbol::new()`. If attributes, arity, symbol_type unknown, use defaults.
  pub fn new(
      name: IString, 
      arity: Arity, 
      attributes : SymbolAttributes, 
      symbol_type: SymbolType,
      sort_spec: SortSpec,
    ) -> Symbol 
  {
    // Compute hash
    static mut SYMBOL_COUNT: u32 = 0;
    unsafe{ SYMBOL_COUNT += 1; }
    let numeric_arity: u32 = if let Arity::Value(v) = arity {
      v as u32
    } else {
      0
    };
    let hash_value = unsafe{ SYMBOL_COUNT } | (numeric_arity << 24); // Maude: self.arity << 24

    let symbol = Symbol{
      name,
      arity,
      attributes,
      symbol_type,
      sort_table: Some(SortTable::new(sort_spec)),
      hash_value
    };

    symbol
  }

  #[inline(always)]
  pub fn with_arity(name: IString, arity: Arity)  -> Symbol {
    Symbol::new(name, arity, SymbolAttributes::default(), SymbolType::default(), SortSpec::Any)
  }

  #[inline(always)]
  pub fn with_name(name: IString)  -> Symbol {
    Symbol::new(name, Arity::None, SymbolAttributes::default(), SymbolType::default(), SortSpec::Any)
  }

  #[inline(always)]
  pub fn is_variable(&self) -> bool {
    self.symbol_type == SymbolType::Variable
  }
  
  #[inline(always)]
  pub fn compare(&self, other: &Symbol) -> std::cmp::Ordering {
    self.hash_value.cmp(&other.hash_value)
  }
  
  pub fn hash(&self) -> u32 {
    self.hash_value
  }
}

impl Display for Symbol {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.arity {
      Arity::Variadic => write!(f, "{}ᵥ", self.name),
      Arity::Value(arity) => write!(f, "{}{}", self.name, int_to_subscript(arity as u32)),
      _ => write!(f, "{}", self.name),
    }
    // write!(f, "{}", self.name)
  }
}

impl Formattable for Symbol {
  fn repr(&self, _style: FormatStyle) -> String {
    // ToDo: Probably defer to `Display` here.
    self.name.to_string()
  }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Hash)]
pub enum SymbolType {
  #[default]
  Standard,
  Variable,
  Operator,
  Data,
  
  True,
  False,
}


#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum SymbolAttribute {
  // Syntactic attributes
  Precedence,
  Gather,
  Format,
  Latex,

  // Semantic attributes
  Strategy,
  Memoized,
  Frozen,
  Constructor,

  // Theory attributes
  Associative,
  Commutative,
  LeftIdentity,
  RightIdentity,
  Idempotent,
  Iterated,
}

pub type SymbolAttributes = BitFlags<SymbolAttribute, u32>;

impl SymbolAttribute {
  //	Conjunctions
  #![allow(non_upper_case_globals)]

  /// Theory Axioms
  pub const Axioms: SymbolAttributes = make_bitflags!(
    SymbolAttribute::{
      Associative
      | Commutative
      | LeftIdentity
      | RightIdentity
      | Idempotent
    }
  );

  pub const Collapse: SymbolAttributes = make_bitflags!(
    SymbolAttribute::{
      LeftIdentity
      | RightIdentity
      | Idempotent
    }
  );

  ///	Simple attributes are just a flag without additional data. They produce a warning if given twice.
  pub const SimpleAttributes: SymbolAttributes = make_bitflags!(
    SymbolAttribute::{
      Associative
      | Commutative
      | Idempotent
      | Memoized
      | Constructor
      | Iterated
    }
  );

  /// All flagged attributes. They need to agree between declarations of an
  /// operator.
  pub const Attributes: SymbolAttributes = make_bitflags!(
    SymbolAttribute::{
      Precedence
      | Gather
      | Format
      | Latex
      | Strategy
      | Memoized
      | Frozen
      | Associative
      | Commutative
      | LeftIdentity
      | RightIdentity
      | Idempotent
      | Iterated
    }
  );
}

