/*!

A `SymbolType` has a `CoreSymbolType` plus additional attributes.

*/

use enumflags2::{bitflags, BitFlags, make_bitflags};

#[derive(Copy, Clone, Eq, PartialEq, Default)]
pub struct SymbolType {
  pub core_type: CoreSymbolType,
  pub attributes: SymbolAttributes
}

/// The most important `CoreSymbolType`s are `Standard` and `Variable`.
///
/// Most of the `CoreSymbolType`s are unimplemented symbol types that are used in Maude.
#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub enum CoreSymbolType {
  #[default]
  Standard,

  // System created symbols
  Variable,
  SortTest,
  InternalTuple,

  // Special properties
  SystemTrue,
  SystemFalse,
  Bubble,

  // Special symbols that do not deal with attachments
  Float,
  String,

  // Special symbols that do deal with attachments
  Branch,
  Equality,
  FloatOp,
  StringOp,
  QuotedIdentifier,
  QuotedIdentifierOp,
  ModelChecker,
  SATSolver,
  MetaLevelOp,
  Loop,
  NaturalNumber, // Succ,
  Minus,
  NumberOp,
  ACUNumberOp,
  CUINumberOp,
  Division,
  RandomOp,
  MatrixOp,
  Counter,
  SocketManager,
  InterpreterManager,
  SMT,
  SMTNumber,
  FileManager,
  StreamManager,
  DirectoryManager,
  ProcessManager,
  TimeManager,
  PRNGManager,
  ObjectConstructor
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

  // OOP attributes
  Config,
  Object,
  Message,
  MsgStatement, // MESSAGE flag was set by msg statement rather than an attribute; only used by SyntacticPreModule

  // Theory attributes
  Associative,
  Commutative,
  LeftIdentity,
  RightIdentity,
  Idempotent,
  Iterated,

  // Misc
  PolymorphicConstant,
  Polymorphic,
  Ditto
}

pub type SymbolAttributes = BitFlags<SymbolAttribute, u32>;

impl SymbolAttribute {
  #![allow(non_upper_case_globals)]

  ///	Conjunctions
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
      | Config
      | Object
      | Message
      | Iterated
      | PolymorphicConstant
    }
  );
  /*
  Non-simple attributes are:
  Precedence, Gather, Format, Latex, Strategy, Frozen, MsgStatement, LeftIdentity, RightIdentity, Polymorphic, Ditto
  */

  /// All flagged attributes except ctor, poly, ditto (and MsgStatement). They need to agree between declarations of an
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
      | Config
      | Object
      | Message
      | Associative
      | Commutative
      | LeftIdentity
      | RightIdentity
      | Idempotent
      | Iterated
      | PolymorphicConstant
    }
  );
}
