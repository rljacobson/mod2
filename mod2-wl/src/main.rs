mod data_atoms;

use mod2_lib::api::symbol::Symbol;
use crate::data_atoms::{FloatAtom, IntegerAtom};

fn main() {
  let int_atom = IntegerAtom::new_atom(43isize);
  let int_symbol: &Symbol = unsafe { &*int_atom.symbol() };

  println!("The data atom is {}.", int_atom);
  println!("Its symbol is {}.", int_symbol);

  let float_atom = FloatAtom::new_atom(std::f64::consts::PI);
  let float_symbol: &Symbol = unsafe { &*float_atom.symbol() };

  println!("The other data atom is {}.", float_atom);
  println!("Its symbol is {}.", float_symbol);
}
