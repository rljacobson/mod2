/*!
Boolean `DagNode` attributes.
*/

use enumflags2::{bitflags, BitFlags, make_bitflags};
use crate::theory::symbol_type::SymbolAttribute;

#[bitflags]
#[repr(u32)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DagNodeAttribute {
  Reduced,      // Reduced up to strategy by equations
  Copied,       // Copied in current copy operation; copyPointer valid
  Unrewritable, // Reduced and not rewritable by rules
  Unstackable,  // Unrewritable and all subterms unstackable or frozen
  Ground,       // No variables occur below this node
  HashValid,    // Node has a valid hash value (storage is theory dependent)
}

pub type DagNodeAttributes = BitFlags<DagNodeAttribute, u32>;

impl DagNodeAttribute {
  #![allow(non_upper_case_globals)]

  // Alias
  // We can share the same bit as UNREWRITABLE for this flag since the rule rewriting strategy that needs UNREWRITABLE
  // never be combined with variant narrowing. Implemented as associated type since Rust does not allow variant aliases.
  pub const IrreducibleByVariantEquations: DagNodeAttribute = DagNodeAttribute::Unrewritable;

  // Conjunction
  pub const RewritingFlags: DagNodeAttributes = make_bitflags!(
    DagNodeAttribute::{
      Reduced | Unrewritable | Unstackable | Ground
    }
  );

  pub fn set_copied_flags(flags: &mut DagNodeAttributes, other_flags: DagNodeAttributes) {
    *flags |= DagNodeAttribute::RewritingFlags & other_flags;
  }
}
