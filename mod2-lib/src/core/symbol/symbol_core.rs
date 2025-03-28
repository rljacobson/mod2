use std::fmt::Display;
use std::sync::atomic::{AtomicU32, Ordering};
use mod2_abs::{int_to_subscript, IString};
use crate::{
  api::Arity,
  core::{
    symbol::{
      SortTable,
      SymbolAttributes,
      SymbolType
    },
    format::{FormatStyle, Formattable},
    strategy::Strategy,
  }
};
use crate::api::symbol::SymbolPtr;
use crate::core::symbol::OpDeclaration;

#[derive(Eq, PartialEq)]
pub struct SymbolCore {
  pub name       : IString,
  pub arity      : Arity,
  pub attributes : SymbolAttributes,
  pub symbol_type: SymbolType,

  pub sort_table: Option<SortTable>,

  /// Unique integer for comparing symbols, also called order. In Maude, the `order`
  /// has lower bits equal to the value of an integer that is incremented every time
  /// a symbol is created and upper 8 bits (bits 24..32) equal to the arity. Note:
  /// We enforce symbol creation with `Symbol::new()` by making hash_value private.
  hash_value : u32,

  // ToDo: Possibly replace with `Option<Box<Strategy>>`, where `None` means "standard strategy".
  // `Strategy`
  pub(crate) strategy: Strategy,
}

// This is an abomination. See `api/built_in/mod.rs`.
unsafe impl Send for SymbolCore {}

unsafe impl Sync for SymbolCore {}

impl SymbolCore {
  /// All symbols must be created with `Symbol::new()`. If attributes, arity, symbol_type unknown, use defaults.
  pub fn new(
      name       : IString,
      arity      : Arity,
      attributes : SymbolAttributes,
      symbol_type: SymbolType,
    ) -> SymbolCore
  {
    // Compute hash
    static SYMBOL_COUNT: AtomicU32 = AtomicU32::new(0);
    SYMBOL_COUNT.fetch_add(1, Ordering::Relaxed);
    let numeric_arity: u32 = arity.as_numeric();
    let hash_value = SYMBOL_COUNT.load(Ordering::Relaxed) | (numeric_arity << 24); // Maude: self.arity << 24

    let symbol = SymbolCore {
      name,
      arity,
      attributes,
      symbol_type,
      sort_table: None,
      hash_value,
      strategy: Default::default(),
    };

    symbol
  }

  #[inline(always)]
  pub fn with_arity(name: IString, arity: Arity)  -> SymbolCore {
    SymbolCore::new(name, arity, SymbolAttributes::default(), SymbolType::default())
  }

  #[inline(always)]
  pub fn with_name(name: IString)  -> SymbolCore {
    SymbolCore::new(name, Arity::None, SymbolAttributes::default(), SymbolType::default())
  }

  #[inline(always)]
  pub fn is_variable(&self) -> bool {
    self.symbol_type == SymbolType::Variable
  }

  #[inline(always)]
  pub fn compare(&self, other: &SymbolCore) -> std::cmp::Ordering {
    self.hash_value.cmp(&other.hash_value)
  }

  #[inline(always)]
  pub fn hash(&self) -> u32 {
    self.hash_value
  }
  
  pub fn add_op_declaration(&mut self, symbol_ptr: SymbolPtr, op_declaration: OpDeclaration) {
    match &mut self.sort_table {
      
      None => {
        let mut sort_table = SortTable::new(symbol_ptr);
        sort_table.add_op_declaration(op_declaration);
        self.sort_table = Some(sort_table);
      }
      
      Some(sort_table) => {
        sort_table.add_op_declaration(op_declaration);
      }
      
    }
  }
  
  #[inline(always)]
  pub(crate) fn arity(&self) -> Arity {
    match self.sort_table {
      None => Arity::Unspecified,
      Some(ref sort_table) => {
        sort_table.arity()
      }
    }
  }
}

impl Display for SymbolCore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.arity {
      Arity::Variadic => write!(f, "{}ᵥ", self.name),
      Arity::Value(arity) => write!(f, "{}{}", self.name, int_to_subscript(arity as u32)),
      _ => write!(f, "{}", self.name),
    }
  }
}

impl Formattable for SymbolCore {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    match style {
      FormatStyle::Debug => {
        write!(f, "SymbolCore<{}>", self.name)
      }
      
      FormatStyle::Simple
      | FormatStyle::Input
      | FormatStyle::Default => {
        write!(f, "{}", self.name)
      }
    }
  }
}