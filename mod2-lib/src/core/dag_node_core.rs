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
  marker::PhantomPinned
};
use std::ptr::null_mut;
use enumflags2::{bitflags, make_bitflags, BitFlags};

use crate::{
  api::{
    Arity,
    dag_node::{
      DagNode,
      DagNodePtr
    },
    symbol::{Symbol, SymbolPtr},
    free_theory::FreeDagNode,
  },
  core::{
    allocator::{
      allocate_dag_node,
    }
  },
};
use crate::api::dag_node::DagNodeVector;

pub type ThinDagNodePtr = *mut DagNodeCore; // A thin pointer to a `DagNodeCore` object.

// ToDo: Isn't this just a tag for what concrete type of the `dyn Trait` that the `DagNodeCore` was created as?
//       If so, can we have a map from `DagNodeKind` to the concrete type?
// ```
// let node: *mut DagNodeCore;
// let dyn_node: *mut dyn DagNode = (&*node).to_dyn_trait();
// let free_node: *mut FreeDagNode = match dyn_node.as_any().downcast_ref::<FreeDagNode>() { Some(n) => n ...}
// ```
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub enum DagNodeTheory {
  #[default]
  Free = 0,
  // ACU,
  // AU,
  // CUI,
  Variable,
  // NA,
  Data,
  // Integer,
  // Float
}


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
  pub(crate) theory_tag: DagNodeTheory,
  pub(crate) flags     : DagNodeFlags,

  // Opt out of `Unpin`
  _pin: PhantomPinned,
}


impl DagNodeCore {
  // region Constructors

  pub fn new(symbol: SymbolPtr) -> DagNodePtr {
    DagNodeCore::with_theory(symbol, DagNodeTheory::default())
  }

  pub fn with_theory(symbol: SymbolPtr, theory: DagNodeTheory) -> DagNodePtr {
    assert!(!symbol.is_null());
    let node     = allocate_dag_node();
    let node_mut = unsafe { &mut *node };

    node_mut.args  = null_mut();
    node_mut.flags = DagNodeFlags::empty();

    if let Arity::Value(arity) = unsafe{ &*symbol }.arity {
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
  pub fn symbol_ref(&self) -> &Symbol {
    unsafe {
      &*self.symbol
    }
  }

  #[inline(always)]
  pub fn arity(&self) -> Arity {
    self.symbol_ref().arity
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
      DagNodeTheory::Free => {
        // Step 1: Create a fake reference to MyStruct
        let fake_ptr: *mut FreeDagNode = std::ptr::null_mut();
        // Step 2: Cast the fake reference to a trait object pointer
        let fake_trait_object: DagNodePtr = fake_ptr as DagNodePtr;
        // Step 3: Extract the vtable from the trait object pointer
        let vtable = std::ptr::metadata(fake_trait_object);
        // Step 4: Combine the thin pointer and vtable pointer into a fat pointer
        let fat_ptr: *mut dyn DagNode = std::ptr::from_raw_parts_mut(thin_dag_node_ptr, vtable);

        fat_ptr
      }
      // DagNodeTheory::Variable => {}
      // DagNodeTheory::Data => {}
      _ => {
        panic!("Thin DagNode has invalid theory tag")
      }
    }
  }

}

impl Display for DagNodeCore {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "node<{}>", self.symbol_ref())
  }
}
