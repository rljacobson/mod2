/*!

The `DagNode` is the heart of the engine. Speed hinges on efficient management of `DagNode` objects. Their creation,
reuse, and destruction are managed by an arena based garbage collecting allocator which relies on the fact that
every `DagNode` is of the same size. Since `DagNode`s can be of different types and have arguments, we make careful use
of transmute and bitflags.

The following compares Maude's `DagNode` to our implementation here.

|                | Maude                                        | mod2lib                  |
|:---------------|:---------------------------------------------|:-------------------------|
| size           | Fixed 3 word size                            | Fixed size struct        |
| tag            | implicit via vtable pointer                  | enum variant             |
| flags          | `MemoryInfo` in first word                   | `BitFlags` field         |
| shared impl    | base class impl                              | enum impl                |
| specialization | virtual function calls                       | match on variant in impl |
| args           | `reinterpret_cast` of 2nd word based on flag | Nested enum              |

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
    dag_node::{
      DagNode,
      DagNodePtr,
      DagNodeVector
    },
    symbol::SymbolPtr,
    free_theory::FreeDagNode,
  },
  core::{
    gc::allocate_dag_node,
    theory::EquationalTheory
  }
};

static FREE_DAG_NODE_VTABLE: DynMetadata<dyn DagNode> = {
  // Create a fake pointer of type `*mut FreeDagNode` (which is concrete)
  let fake_ptr: *mut FreeDagNode = std::ptr::null_mut();
  // Cast it to a trait object pointer; this creates a fat pointer with the vtable for `FreeDagNode`
  let fake_trait_object: *mut dyn DagNode = fake_ptr as *mut dyn DagNode;
  // This prevents the compiler from optimizing the vtable away
  _ = fake_trait_object.is_null();
  // Extract the metadata (the vtable pointer)
  std::ptr::metadata(fake_trait_object)
};


pub type ThinDagNodePtr = *mut DagNodeCore; // A thin pointer to a `DagNodeCore` object.


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


pub struct DagNodeCore {
  pub(crate) symbol    : SymbolPtr,
  // ToDo: Figure out `args` representation at `DagNodeCore` level.
  /// Either null or a pointer to a `GCVector<T>`.
  ///
  /// The problem with having an `args` member on `DagNodeCore` is that different theories will store different
  /// types in `args`, like `(DagNodePtr, Multiplicity)`. The low-level `args` details can be shifted to
  /// the theory node types, but then every theory would need to reimplement them. Likewise with `mark()` and
  /// the destructor.
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

  pub fn with_theory(symbol: SymbolPtr, theory: EquationalTheory) -> DagNodePtr {
    let node     = allocate_dag_node();
    let node_mut = unsafe { &mut *node };

    node_mut.args  = std::ptr::null_mut();
    node_mut.flags = DagNodeFlags::empty();

    if let Arity::Value(arity) = symbol.arity() {
      if arity > 1 {
        let vec = DagNodeVector::with_capacity(arity as usize);
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

  #[inline(always)]
  pub fn simple_reuse(&self) -> bool {
    !self.flags.contains(DagNodeFlag::Marked) && !self.needs_destruction()
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
        // Step 1: Create a fake reference to MyStruct
        // let fake_ptr: *mut FreeDagNode = std::ptr::null_mut();
        // // Step 2: Cast the fake reference to a trait object pointer
        // let fake_trait_object: *mut dyn DagNode = fake_ptr as *mut dyn DagNode;
        // // Step 3: Extract the vtable from the trait object pointer
        // let vtable = std::ptr::metadata(fake_trait_object);
        // Step 4: Combine the thin pointer and vtable pointer into a fat pointer
        // let fat_ptr: *mut dyn DagNode = std::ptr::from_raw_parts_mut(thin_dag_node_ptr, vtable);
        let fat_ptr: *mut dyn DagNode = std::ptr::from_raw_parts_mut(thin_dag_node_ptr, FREE_DAG_NODE_VTABLE);
        if fat_ptr.is_null() {
          panic!("FreeDagNodePtr could not be created from ThinDagNodePtr");
        }

        DagNodePtr::new(fat_ptr)
      }
      // EquationalTheory::Variable => {}
      // EquationalTheory::Data => {}
      _ => {
        panic!("Thin DagNode has invalid theory tag")
      }
    }
  }

}

impl Display for DagNodeCore {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "node<{}>", self.symbol())
  }
}
