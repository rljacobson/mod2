/*!

Definitions related to symbols. Symbols can be thought of as names to which additional information is attached, such as
arity and theory axioms.

In an expression like, `f[x, Thing1, 45]`, the symbols are `f`, `x`, and `Thing1`.

*/

mod symbol_core;
mod sort_table;
pub(self) mod op_declaration;

pub use symbol_core::SymbolCore;
pub use sort_table::{SortTable, BxSortTable};
pub use op_declaration::*;


use enumflags2::{bitflags, make_bitflags, BitFlags};

#[derive(Copy, Clone, Eq, PartialEq, Default, Debug, Hash)]
pub enum SymbolType {
  #[default]
  Standard,
  Variable,
  Operator,
  Data,

  // Built-in Data Types
  True,
  False,
  String,
  Float,
  Integer,
  NaturalNumber,
}

impl SymbolType {
  pub fn is_build_in_data_type(&self) -> bool {
    match self {
      SymbolType::True 
      | SymbolType::False 
      | SymbolType::String 
      | SymbolType::Float 
      | SymbolType::Integer 
      | SymbolType::NaturalNumber => true,
      
      _ => false
    }
  }
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
