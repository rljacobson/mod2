use crate::theory::symbol::TheorySymbol;

#[derive(Copy, Clone, Default)]
pub struct FreeSymbol {
  // These members are just placeholders for now.
  discrimination_net: u8,
  strategy: u8,
}

impl TheorySymbol for FreeSymbol {

}

