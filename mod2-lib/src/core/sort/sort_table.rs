/*!

Combines the function of SortTable and SortConstraintTable in Maude.

*/

use crate::core::sort::SortPtr;

#[derive(Eq, PartialEq)]
pub struct SortTable{
  sort: SortPtr,
}

impl SortTable {
  pub fn new(sort: SortPtr) -> Self {
    SortTable{ sort }
  }
}
