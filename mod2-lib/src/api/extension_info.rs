use enumflags2::{bitflags, BitFlags};
use crate::{
  api::{ArgIndex, DagNodePtr, Multiplicity},
  core::gc::root_container::RootVec
};

pub type MaybeExtensionInfo = Option<ExtensionInfoPtr>;
pub type ExtensionInfoPtr   = UnsafePtr<ExtensionInfo>;

#[derive(Clone, PartialEq)]
pub enum ExtensionInfo {
  AUExtension{
    subject   : DagNodePtr,
    first     : ArgIndex,
    last      : ArgIndex,
    attributes: ExtensionAttributes,
  },

  ACUExtension{
    subject               : DagNodePtr,
    unmatched             : Box<RootVec>,
    unmatched_multiplicity: Vec<Multiplicity>,
    attributes            : ExtensionAttributes,
  },

  SExtension{
    attributes: ExtensionAttributes,
  }
}

impl ExtensionInfo {

  #[inline(always)]
  fn attribute(&self, attribute: ExtensionAttribute) -> bool {
    match self {
      ExtensionInfo::AUExtension{ attributes, ..}
      | ExtensionInfo::ACUExtension { attributes, .. }
      | ExtensionInfo::SExtension { attributes, .. } => {
        attributes.contains(attribute)
      }
    }
  }

  #[inline(always)]
  fn set_attribute(&mut self, attribute: ExtensionAttribute, value: bool) {
    match self {
      ExtensionInfo::AUExtension{ attributes, ..}
      | ExtensionInfo::ACUExtension { attributes, .. }
      | ExtensionInfo::SExtension { attributes, .. } => {
        if value {
          attributes.insert(attribute);
        } else {
          attributes.remove(attribute);
        }
      }
    }
  }

  /// The match phase records if extension info is valid after the match
  /// phase or do we need to wait until after the solve phase is successful.
  pub fn valid_after_match(&self) -> bool {
    self.attribute(ValidAfterMatch)
  }

  /// sets the valid_after_match field
  pub fn set_valid_after_match(&mut self, value: bool) {
    self.set_attribute(ValidAfterMatch, value);
  }

  /// Did we match the whole of the subject theory layer (extension is empty) or just part.
  pub fn matched_whole(&self) -> bool {
    self.attribute(MatchedWhole)
  }

  /// sets the matched_whole field
  pub fn set_matched_all(&mut self, value: bool) {
    self.set_attribute(MatchedWhole, value);
  }

  pub fn build_matched_portion(&self) -> DagNodePtr {
    unimplemented!();
    // match self {
    //   ExtensionInfo::AUExtension { .. } => {}
    //   ExtensionInfo::ACUExtension { .. } => {}
    //   ExtensionInfo::SExtension { .. } => {}
    //
    //   _ => panic!("{:?} is not an ExtensionInfo", self)
    // }
  }
}

#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ExtensionAttribute {
  /// Indicates whether the extension information is valid immediately after the match phase.
  /// If `false`, the extension information must be recomputed or validated after the solve phase.
  ValidAfterMatch,
  /// We matched the whole of the subject theory layer (extension is empty).
  MatchedWhole,

  // AU Theory
  /// The portion matched contains an identity not present in subject.
  ExtraIdentity,
}

use ExtensionAttribute::*;
use mod2_abs::UnsafePtr;

pub type ExtensionAttributes = BitFlags<ExtensionAttribute, u8>;

