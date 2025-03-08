/*!

Combines the function of SortTable and SortConstraintTable in Maude.

*/

use crate::core::sort::sort_spec::SortSpec;

#[derive(Eq, PartialEq, Hash)]
pub(crate) struct SortTable{
  sort_spec: SortSpec,
}

impl SortTable {
  pub fn new(sort_spec: SortSpec) -> Self {
    SortTable{ sort_spec }
  }
}