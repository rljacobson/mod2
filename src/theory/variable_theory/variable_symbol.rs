use std::marker::PhantomData;

use crate::abstractions::IString;
use crate::theory::symbol::{Symbol, TheorySymbol};

#[derive(Copy, Clone, Default)]
pub struct VariableSymbol {
  phantom_data: PhantomData<u8>
}

impl TheorySymbol for VariableSymbol {}
