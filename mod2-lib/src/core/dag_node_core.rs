/*!

The [`DagNodeCore`] is the heart of the engine. Speed hinges on efficient management
of [`DagNodeCore`] objects. Their creation, reuse, and destruction are managed by
an arena based garbage collecting allocator which relies on the fact that every
implementor of [`DagNode`] is of the same size. Since [`DagNode`]s can be of
different types and have arguments, we make careful use of transmute and bitflags.

The following compares Maude's [`DagNode`] to our implementation here.

|                | Maude                                        | mod2lib                     |
|:---------------|:---------------------------------------------|:----------------------------|
| size           | Fixed 3 word size (or 6 words?)              | Fixed size struct (4 words) |
| tag            | implicit via vtable pointer                  | enum variant                |
| flags          | `MemoryInfo` in first word                   | `BitFlags` field            |
| shared impl    | base class impl                              | enum impl                   |
| specialization | virtual function calls                       | match on variant in impl    |
| args           | `reinterpret_cast` of 2nd word based on flag | Nested enum                 |

*/

use std::{
  fmt::{Display, Formatter},
  marker::PhantomPinned,
  ptr::DynMetadata
};
use enumflags2::{bitflags, make_bitflags, BitFlags};

use crate::{
  api::{
    Arity,
    built_in::{
      StringBuiltIn,
      BoolDagNode,
      FloatDagNode,
      IntegerDagNode,
      NaturalNumberDagNode,
      StringDagNode,
    },
    dag_node::{
      DagNode,
      DagNodePtr,
      DagNodeVector
    },
    free_theory::FreeDagNode,
    symbol::SymbolPtr,
    variable_theory::VariableDagNode,
  },
  core::{
    gc::allocate_dag_node,
    theory::EquationalTheory
  }
};

/// Create vtable pointers for concrete `DagNode` types
macro_rules! make_dag_node_vtable {
    ($tablename:ident, $nodetype:ty) => {
      static $tablename: DynMetadata<dyn DagNode> = {
        // Create a fake pointer of type `*mut FreeDagNode` (which is concrete)
        let fake_ptr: *mut $nodetype = std::ptr::null_mut();
        // Cast it to a trait object pointer; this creates a fat pointer with the vtable for `FreeDagNode`
        let fake_trait_object: *mut dyn DagNode = fake_ptr as *mut dyn DagNode;
        // This prevents the compiler from optimizing the vtable away
        _ = fake_trait_object.is_null();
        // Extract the metadata (the vtable pointer)
        std::ptr::metadata(fake_trait_object)
      };
    };
}

make_dag_node_vtable!(FREE_DAG_NODE_VTABLE, FreeDagNode);
make_dag_node_vtable!(VARIABLE_DAG_NODE_VTABLE, VariableDagNode);
make_dag_node_vtable!(BOOL_DAG_NODE_VTABLE, BoolDagNode);
make_dag_node_vtable!(FLOAT_DAG_NODE_VTABLE, FloatDagNode);
make_dag_node_vtable!(INTEGER_DAG_NODE_VTABLE, IntegerDagNode);
make_dag_node_vtable!(NATURALNUMBER_DAG_NODE_VTABLE, NaturalNumberDagNode);
make_dag_node_vtable!(STRING_DAG_NODE_VTABLE, StringDagNode);


/// A thin pointer to a `DagNodeCore` object
pub type ThinDagNodePtr = *mut DagNodeCore;

/// The `DagNodeCore::inline` field needs to be large enough to hold the largest data value that will be stored there.
/// The largest value is a `Vec<u8>` or `String` (which are the same size), 24 bytes on most 64-bit systems.
// ToDo: Replace this with a const max of all sizes.
const INLINE_BYTE_COUNT: usize = size_of::<StringBuiltIn>();


#[bitflags]
#[repr(u8)]
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum DagNodeFlag {
  /// Marked as in use
  Marked,
  /// Has args that need destruction
  NeedsDestruction,
  /// Reduced up to strategy by equations
  Reduced,
  /// Copied in current copy operation; copyPointer valid
  Copied,
  /// Reduced and not rewritable by rules
  Unrewritable,
  /// Unrewritable and all subterms unstackable or frozen
  Unstackable,
  /// No variables occur below this node
  GroundFlag,
  /// Node has a valid hash value (storage is theory dependent)
  HashValid,
}

impl DagNodeFlag {
  #![allow(non_upper_case_globals)]

  /// An alias - We can share the same bit for this flag since the rule rewriting
  /// strategy that needs `Unrewritable` will never be combined with variant narrowing.
  pub const IrreducibleByVariantEquations: DagNodeFlag = DagNodeFlag::Unrewritable;

  // Conjunctions

  /// Flags for rewriting
  pub const RewritingFlags: DagNodeFlags = make_bitflags!(
    DagNodeFlag::{
      Reduced | Unrewritable | Unstackable | GroundFlag
    }
  );
}

pub type DagNodeFlags = BitFlags<DagNodeFlag, u8>;

/// `DagNodeCore` reserves a storage area that is sized to accommodate
/// the largest `DagNode` implementor. Instead of a generic memory
/// pool, we specify common fields and two polymorphic fields:
///
///   1. an `args` pointer to garbage collected memory;
///   2. 16 bytes of "inline" memory for storing values.
///
/// Any derived DagNode instance must only use these fields and is
/// responsible for reinterpreting data stored there for its own needs.
/// This is how we ensure all implementors of `DagNode` are the same size.
///
/// Keep the fields ordered to optimize memory layout.
pub struct DagNodeCore {
  pub(crate) symbol: SymbolPtr,

  /// Several Theories store values inline:
  ///   - `NADataType` values for `NADagNode<T: NADataType>`, usually 64 bits
  ///   - A fat pointer for `ACUDagNode::runs_buffer: Vec<usize>`
  ///   - `IString` for `VariableDagNode`
  ///
  /// You can store and retrieve raw data from this field by emulating this example from `VariableDagNode`:
  /// ```ignore
  /// #[inline(always)]
  ///  pub fn index(&self) -> VariableIndex {
  ///    unsafe {
  ///      let ptr = self.core().inline.as_ptr().add(VARIABLE_INDEX_OFFSET) as *const VariableIndex;
  ///      std::ptr::read_unaligned(ptr)
  ///    }
  ///  }
  ///
  ///  #[inline(always)]
  ///  pub fn set_index(&mut self, index: VariableIndex) {
  ///    unsafe {
  ///      let ptr = self.core_mut().inline.as_mut_ptr().add(VARIABLE_INDEX_OFFSET) as *mut VariableIndex;
  ///      std::ptr::write_unaligned(ptr, index);
  ///    }
  ///  }
  /// ```
  // ToDo: Can we use the `args` field for this purpose?
  pub(crate) inline: [u8; INLINE_BYTE_COUNT],

  // ToDo: Figure out `args` representation at `DagNodeCore` level.
  /// Either null or a pointer to a `GCVector<T>`.
  ///
  /// The problem with having an `args` member on `DagNodeCore` is that different theories will store different
  /// types in `args`, like `(DagNodePtr, Multiplicity)`. The low-level `args` details can be shifted to
  /// the theory node types, but then every theory would need to reimplement them. Likewise with `mark()` and
  /// the destructor/finalizer.
  pub(crate) args      : *mut u8,
  pub(crate) sort_index: i8, // sort index within kind
  pub(crate) theory_tag: EquationalTheory,
  pub(crate) flags     : DagNodeFlags,

  // Opt out of `Unpin`
  _pin: PhantomPinned,
}


impl DagNodeCore {
  // region Constructors

  pub fn new(symbol: SymbolPtr) -> DagNodePtr {
    DagNodeCore::with_theory(symbol, EquationalTheory::default())
  }

  // ToDo: The theory should be deducible from the symbol.
  pub fn with_theory(symbol: SymbolPtr, theory: EquationalTheory) -> DagNodePtr {
    let node     = allocate_dag_node();
    let node_mut = unsafe { &mut *node };

    // Re-initialize memory
    node_mut.args   = std::ptr::null_mut();
    node_mut.flags  = DagNodeFlags::empty();
    node_mut.inline = [0; 24];

    // ToDo: Improve API for args. E.g. make `DagNodeVector` generic.
    if let Arity::Value(arity) = symbol.arity() {
      if arity > 1 {
        // ToDo: The vector probably shouldn't be allocated here.
        let vec       = DagNodeVector::with_capacity(arity as usize);
        node_mut.args = (vec as *mut DagNodeVector) as *mut u8;
        node_mut.flags.insert(DagNodeFlag::NeedsDestruction);
      }
    };

    node_mut.theory_tag = theory;
    node_mut.symbol     = symbol;

    DagNodeCore::upgrade(node)
  }

  // endregion Constructors

  // region Accessors

  #[inline(always)]
  pub fn symbol(&self) -> SymbolPtr {
    self.symbol
  }

  #[inline(always)]
  pub fn arity(&self) -> Arity {
    self.symbol().arity()
  }

  // endregion

  // region GC related methods
  #[inline(always)]
  pub fn is_marked(&self) -> bool {
    self.flags.contains(DagNodeFlag::Marked)
  }

  #[inline(always)]
  pub fn needs_destruction(&self) -> bool {
    self.flags.contains(DagNodeFlag::NeedsDestruction)
  }

  //endregion

  /// Upgrades the thin pointer to a DagNodeCore object to a fat pointer to a concrete implementor of the `DagNode`
  /// trait, returning a fat pointer to a `dyn DagNode` with the correct vtable. The concrete type is selected based
  /// on `DagNodeCore::theory_tag`.
  ///
  /// This is a huge pain to do.
  #[inline(always)]
  pub fn upgrade(thin_dag_node_ptr: ThinDagNodePtr) -> DagNodePtr {
    assert!(!thin_dag_node_ptr.is_null());
    match unsafe { thin_dag_node_ptr.as_ref_unchecked().theory_tag } {

      EquationalTheory::Free => {
        let fat_ptr: *mut dyn DagNode = std::ptr::from_raw_parts_mut(thin_dag_node_ptr, FREE_DAG_NODE_VTABLE);
        debug_assert!(!fat_ptr.is_null(), "FreeDagNodePtr could not be created from ThinDagNodePtr");
        DagNodePtr::new(fat_ptr)
      }

      EquationalTheory::Variable => {
        let fat_ptr: *mut dyn DagNode = std::ptr::from_raw_parts_mut(thin_dag_node_ptr, VARIABLE_DAG_NODE_VTABLE);
        debug_assert!(!fat_ptr.is_null(), "VariableDagNodePtr could not be created from ThinDagNodePtr");
        DagNodePtr::new(fat_ptr)
      }

      EquationalTheory::Bool => {
        let fat_ptr: *mut dyn DagNode = std::ptr::from_raw_parts_mut(thin_dag_node_ptr, BOOL_DAG_NODE_VTABLE);
        debug_assert!(!fat_ptr.is_null(), "BoolDagNodePtr could not be created from ThinDagNodePtr");
        DagNodePtr::new(fat_ptr)
      }

      EquationalTheory::Float => {
        let fat_ptr: *mut dyn DagNode = std::ptr::from_raw_parts_mut(thin_dag_node_ptr, FLOAT_DAG_NODE_VTABLE);
        debug_assert!(!fat_ptr.is_null(), "FloatDagNodePtr could not be created from ThinDagNodePtr");
        DagNodePtr::new(fat_ptr)
      }

      EquationalTheory::Integer => {
        let fat_ptr: *mut dyn DagNode = std::ptr::from_raw_parts_mut(thin_dag_node_ptr, INTEGER_DAG_NODE_VTABLE);
        debug_assert!(!fat_ptr.is_null(), "IntegerDagNodePtr could not be created from ThinDagNodePtr");
        DagNodePtr::new(fat_ptr)
      }

      EquationalTheory::NaturalNumber => {
        let fat_ptr: *mut dyn DagNode = std::ptr::from_raw_parts_mut(thin_dag_node_ptr, NATURALNUMBER_DAG_NODE_VTABLE);
        debug_assert!(!fat_ptr.is_null(), "NaturalNumberDagNodePtr could not be created from ThinDagNodePtr");
        DagNodePtr::new(fat_ptr)
      }

      EquationalTheory::String => {
        let fat_ptr: *mut dyn DagNode = std::ptr::from_raw_parts_mut(thin_dag_node_ptr, STRING_DAG_NODE_VTABLE);
        debug_assert!(!fat_ptr.is_null(), "StringDagNodePtr could not be created from ThinDagNodePtr");
        DagNodePtr::new(fat_ptr)
      }

      // _ => {
      //   panic!("Thin DagNode has invalid theory tag")
      // }
    }
  }

  pub fn finalize_in_place(&mut self) {
    // ToDo: We need a better way of finalizing dag nodes
    let mut dag_node = DagNodeCore::upgrade(self);
    dag_node.finalize();
  }

}

impl Display for DagNodeCore {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "node<{}>", self.symbol())
  }
}
