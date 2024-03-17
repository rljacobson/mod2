/*!

Symbols belong to some equational theory. The theory may be associative, commutative, have an identity (unity), and so
forth. This module allows the conversion between sets of attributes and equational theories.

*/

use std::convert::Into;
use enumflags2::{BitFlag, BitFlags, bitflags, ConstToken, make_bitflags};
use crate::theory::Theory::Variable;
// use crate::theory::theory_attributes::TheoryAttribute::{Associative, Commutative, Idempotent, LeftIdentity, RightIdentity};


/// Theory attributes determine which equational theory a symbol lives in.

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TheoryAttribute {
  Associative,
  Commutative,
  Idempotent,
  Iterated,
  LeftIdentity,
  RightIdentity,
  Variable,
  Constructor,
}

pub(crate) type TheoryAttributes = BitFlags<TheoryAttribute>;

impl TheoryAttribute {
  #![allow(non_upper_case_globals)]
  /// Identity / Unity
  pub const Identity: TheoryAttributes = make_bitflags!( TheoryAttribute::{LeftIdentity | RightIdentity});
  /// Associative Commutative with Unity
  pub const ACU     : TheoryAttributes = make_bitflags!(TheoryAttribute::{ Associative | Commutative | LeftIdentity | RightIdentity });
  /// Associative with Unity
  pub const AU      : TheoryAttributes = make_bitflags!(TheoryAttribute::{ Associative | LeftIdentity | RightIdentity });
  /// Commutative with Unity, Idempotent
  pub const CUI     : TheoryAttributes = make_bitflags!(TheoryAttribute::{ Commutative | LeftIdentity | RightIdentity | Idempotent });
  /// Free (of constraints)
  pub const Free    : TheoryAttributes = unsafe{TheoryAttributes::from_bits_unchecked_c(0, BitFlags::CONST_TOKEN)};
}


/// These are the equational theories with explicit matching/unification implementations in Maude.
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub(crate) enum Theory {
  ACU, // Associative Commutative with Unity
  AU, // Associative with Unity
  CUI, // Commutative with Unity, Idempotent?
  #[default]
  Free, // Free
  NA, // Non-Algebraic
  S, // Successor (Peano Arithmetic)
  Variable

  /*
  Some associative theories listed in Maude:
    A, AUl, AUr, AU, AC, ACU, and ConfigSymbol

  Some non-associative theories listed in Maude:
    C, CU, Ul, Ur, CI, CUI, UlI, UrI, and CUI_NumberOpSymbol
  */
}

impl Theory {

  /// Returns either ACU, AU, CUI, Free, or Variable according to whether the attribute set contains Associative and/or
  /// Commutative. The caller is responsible for determining that none of the other theories applies.
  pub fn from_attributes(attributes: TheoryAttributes) -> Self {
    use TheoryAttribute::*;

    // Variable trumps all.
    if attributes.contains(Variable) {
      Theory::Variable
    } else if attributes.contains(Associative) {
      if attributes.contains(Commutative) {
        Theory::ACU
      } else {
        Theory::AU
      }
    } else if attributes.contains(Commutative) {
      Theory::CUI
    } else {
      Theory::Free
    }
  }
}
