/*!

To allow for sharing of common subexpressions (Cons hashing), terms are transformed into a directed acyclic graph (DAG).

*/

use mod2_abs::RcCell;

use crate::{
  theory::{
    dag_node_attributes::DagNodeAttributes,
    symbol::SymbolPtr
  }
};

pub type RcDagNode = RcCell<DagNode>;
pub type NodeList  = Vec<RcDagNode>;

#[derive(Clone)]
pub struct DagPair {
  pub(crate) dag_node:     RcDagNode,
  pub(crate) multiplicity: u32,
}

pub struct DagNode {
  pub(crate) top_symbol: SymbolPtr,
  pub(crate) args:       NodeList,
  pub(crate) attributes: DagNodeAttributes,
  pub(crate) sort_index: i32,
  pub(crate) hash:       u32,
}

impl DagNode {
  /// Returns an iterator over `(RcDagNode, u32)` pairs for the arguments.
  #[inline(always)]
  fn iter_args(&self) -> Box<dyn Iterator<Item = RcDagNode> + '_> {
    Box::new(self.args.iter().cloned())
  }


}
