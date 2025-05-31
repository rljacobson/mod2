/*!

The `DagNode` trait is the interface all DAG node's must implement.

Requirements of implementers of `DagNode`:
 1. DAG nodes should be newtypes of `DagNodeCore`. In particular...
 2. DAG nodes *must* have the same memory representation as a `DagNodeCore`.
 3. Implementers of `DagNode` are responsible for casting pointers, in particular its arguments.
 4. If an implementor owns resources, including ref counted objects like `IString`, it must provide an implementation
    of `DagNode::finalize()`. It must also set its `NeedsDestruction` flag.
 5. If an implementor holds no children, or if its children are represented differently than just `DagNodePtr`, it
    must provide an implementation of `iter_args`, `insert_child`, `len`, `compare_arguments`, `mark`.

*/

use std::{
  any::Any,
  cmp::{max, Ordering},
  fmt::Display,
  hash::{Hash, Hasher},
  iter::Iterator,
  ops::Deref,
  sync::atomic::Ordering::Relaxed,
};
use mod2_abs::UnsafePtr;
use crate::{
  api::{
    symbol::SymbolPtr,
    Arity,
    term::Term
  },
  core::{
    dag_node_core::{
      DagNodeCore,
      DagNodeFlag,
      DagNodeFlags,
      ThinDagNodePtr
    },
    format::{
      FormatStyle,
      Formattable
    },
    gc::{
      node_allocator::ACTIVE_NODE_COUNT,
      gc_vector::{
        GCVector,
        GCVectorRefMut
      }
    },
    sort::SortPtr,
    HashConsSet
  },
  impl_display_debug_for_formattable,
  HashType,
};

// A fat pointer to a trait object. For a thin pointer to a DagNodeCore, use ThinDagNodePtr
pub type DagNodePtr          = UnsafePtr<dyn DagNode + 'static>;
pub type DagNodeVector       = GCVector<DagNodePtr>;
pub type DagNodeVectorRefMut = GCVectorRefMut<DagNodePtr>;

/// Commutative theories can have this more compact representation
#[derive(Copy, Clone)]
pub struct DagPair {
  pub(crate) dag_node    : DagNodePtr,
  pub(crate) multiplicity: u8,
}


pub trait DagNode {

  fn as_any(&self) -> &dyn Any;
  // {
  //   self
  // }

  fn as_any_mut(&mut self) -> &mut dyn Any;
  // {
  //   self
  // }

  fn as_ptr(&self) -> DagNodePtr;
  // {
  //   DagNodePtr::new(self as *const dyn DagNode as *mut dyn DagNode)
  // }

  /// Gives the same value as the corresponding method on `Term`, but computes it anew each time its called.
  /// This should be kept in sync with the theory's term structural hash computation.
  fn structural_hash(&self) -> HashType;

  // region Accessors

  /// Trait level access to members for shared implementation
  fn core(&self) -> &DagNodeCore;
  fn core_mut(&mut self) -> &mut DagNodeCore;

  #[inline(always)]
  fn arity(&self) -> Arity {
    self.core().arity()
  }

  /// MUST override if `Self::args` is not a [`DagNodeVector`].
  /// Implement an empty iterator with:
  ///      `Box::new(std::iter::empty::<DagNodePtr>())`
  fn iter_args(&self) -> Box<dyn Iterator<Item=DagNodePtr>> {

    // The empty case
    if self.core().args.is_null() {
      assert!(
        match self.arity() {
          Arity::Value(v) if v > 0 => false,
          _ => true,
        }
      );
      Box::new(std::iter::empty())
    }
    // The vector case
    // (other reasons for `needs_destruction` have null args)
    else if self.core().needs_destruction() {
      assert!( !self.symbol().is_variable() );
      assert!(
        match self.arity() {
          Arity::Value(v) if v > 1 => true,

          Arity::Variadic
          | Arity::Any => true,

          _ => {
            println!("Arity of node is {:?}", self.arity());
            false
          },
        }
      );

      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);
      Box::new(node_vector.iter().cloned())
    }
    // The singleton case
    else {
      assert!( !self.symbol().is_variable() );
      assert!(
        match self.arity() {
          Arity::Value(v) if v == 1 => true,

          Arity::Variadic
          | Arity::Any => true,

          _ => false,
        }
      );

      let node = arg_to_dag_node(self.core().args);

      // Make a fat pointer to the single node and return an iterator to it. This allows `self` to
      // escape the method. Of course, `self` actually points to a `DagNode` that is valid for the
      // lifetime of the program, so even in the event of the GC equivalent of a dangling pointer
      // or use after free, this will be safe.
      let v = unsafe { std::slice::from_raw_parts(&node, 1) };
      Box::new(v.iter().map(|n| *n))
    }
  }

  /// MUST override if `Self::args` is not a [`DagNodeVector`]
  fn insert_child(&mut self, new_child: DagNodePtr){
    // ToDo: Should we signal if arity is exceeded and/or DagNodeVector needs to reallocate?

    // Empty case
    if self.core().args.is_null() {
      self.core_mut().args = new_child.as_mut_ptr() as *mut u8;
    }
    // Vector case
    // (other reasons for `needs_destruction` have null args)
    else if self.core().needs_destruction() {
      let node_vec: DagNodeVectorRefMut = arg_to_node_vec(self.core_mut().args);
      node_vec.push(new_child)
    }
    // Singleton case
    else {
      let existing_child = arg_to_dag_node(self.core_mut().args);
      let arity = if let Arity::Value(arity) = self.arity() {
        max(arity, 2)
      } else {
        2
      };
      let node_vec   = DagNodeVector::with_capacity(arity as usize);

      node_vec.push(existing_child);
      node_vec.push(new_child);

      // Take ownership
      self.set_flags(DagNodeFlag::NeedsDestruction.into());
      self.core_mut().args = (node_vec as *mut DagNodeVector) as *mut u8;
    }
  }


  /// Gives the top symbol of this term.
  #[inline(always)]
  fn symbol(&self) -> SymbolPtr {
    self.core().symbol
  }

  // ToDo: Implement DagNodeCore::get_sort() when `SortTable` is implemented.
  #[inline(always)]
  fn get_sort(&self) -> Option<SortPtr> {
    unimplemented!()
    /*
    let sort_index: i8 = self.sort_index();
    match sort_index {
      n if n == SpecialSort::Unknown as i8 => None,

      // Anything else
      sort_index => {
        self
            .dag_node_members()
            .top_symbol
            .sort_table()
            .range_component()
            .borrow()
            .sort(sort_index)
            .upgrade()
      }
    }
    */
  }


  #[inline(always)]
  fn set_sort_index(&mut self, sort_index: i8) {
    self.core_mut().sort_index = sort_index;
  }


  #[inline(always)]
  fn sort_index(&self) -> i8 {
    self.core().sort_index
  }


  /// Set the sort to best of original and other sorts
  #[inline(always)]
  fn upgrade_sort_index(&mut self, other: DagNodePtr) {
    //  We set the sort to best of original and other sorts; that is:
    //    SORT_UNKNOWN, SORT_UNKNOWN -> SORT_UNKNOWN
    //    SORT_UNKNOWN, valid-sort -> valid-sort
    //    valid-sort, SORT_UNKNOWN -> valid-sort
    //    valid-sort,  valid-sort -> valid-sort
    //
    //  We can do it with a bitwise AND trick because valid sorts should
    //  always be in agreement and SORT_UNKNOWN is represented by -1, i.e.
    //  all 1 bits.
    self.set_sort_index(self.sort_index() & other.sort_index())
  }


  /// MUST be overriden if `Self::args` is not a `DagNodeVec`
  fn len(&self) -> usize {
    // The empty case
    if self.core().args.is_null() {
      0

    } // The vector case
    else if self.core().needs_destruction() {
      // We need to allow `self` to escape the method, same as `Single(..)` branch.
      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);

      node_vector.len()

    } // The singleton case
    else {
      1
    }
  }


  #[inline(always)]
  fn flags(&self) -> DagNodeFlags {
    self.core().flags
  }

  #[inline(always)]
  fn set_reduced(&mut self) {
    self.core_mut().flags.insert(DagNodeFlag::Reduced);
  }

  #[inline(always)]
  fn is_reduced(&self) -> bool {
    self.core().flags.contains(DagNodeFlag::Reduced)
  }

  #[inline(always)]
  fn set_flags(&mut self, flags: DagNodeFlags) {
    self.core_mut().flags.insert(flags);
  }

  // endregion Accessors

  // region Comparison

  /// Defines a partial order on `DagNode`s by comparing the symbols and the arguments recursively.
  fn compare(&self, other: DagNodePtr) -> Ordering {
    let symbol_order = self.symbol().compare(&*other.symbol());

    match symbol_order {
      Ordering::Equal => self.compare_arguments(other),
      _ => symbol_order,
    }
  }

  /// MUST be overridden is `Self::args` something other than a `DagNodeVector`.
  fn compare_arguments(&self, other: DagNodePtr) -> Ordering {
    let symbol = self.symbol();

    assert_eq!(symbol, other.symbol(), "symbols differ");

    if other.core().theory_tag != self.core().theory_tag {
      // if let None = other.as_any().downcast_ref::<FreeDagNode>() {}
      // Not even the same theory. It's not clear what to return in this case, so just compare symbols.
      return symbol.compare(&*other.symbol());
    };

    if (true, true) == (self.core().args.is_null(), other.core().args.is_null()) {
      return Ordering::Equal;
    }
    else if (false, false) == (self.core().args.is_null(), other.core().args.is_null()) {
      if (false, false) == (self.core().needs_destruction(), other.core().needs_destruction()) {
        // Singleton case
        let self_child     : DagNodePtr = arg_to_dag_node(self.core().args);
        let other_child_ptr: DagNodePtr = arg_to_dag_node(other.core().args);

        // Fast bail on equal pointers.
        if self_child.addr_eq(other_child_ptr){
          return Ordering::Equal; // Points to same node
        }

        return self_child.compare(other_child_ptr);
      }
      else if (true, true) == (self.core().needs_destruction(), other.core().needs_destruction()) {
        // The vector case
        let self_arg_vec : &DagNodeVector = arg_to_node_vec(self.core().args);
        let other_arg_vec: &DagNodeVector = arg_to_node_vec(other.core().args);

        // ToDo: This check isn't in Maude?
        if self_arg_vec.len() != other_arg_vec.len() {
          return if self_arg_vec.len() > other_arg_vec.len() {
            Ordering::Greater
          } else {
            Ordering::Less
          };
        }

        // Compare all children from left to right
        // Maude structures this so that it's tail call optimized, but we don't have that guarantee.
        for (&p, &q) in self_arg_vec.iter().zip(other_arg_vec.iter()) {
          // Fast bail on equal pointers.
          if p.addr_eq(q) {
            continue; // Points to same node
          }

          let result = p.compare(q);

          if result.is_ne() {
            return result;
          }
        }
      }
    }
    else {
      // It's not clear what to do in this case, if the case can even happen.
      if other.core().args.is_null() {
        return Ordering::Greater;
      } else {
        return Ordering::Less;
      }
    }

    // Survived all attempts at finding inequality.
    Ordering::Equal
  }

  /// Checks pointer equality first, then compares symbols for equality recursively.
  fn equals(&self, other: DagNodePtr) -> bool {
    std::ptr::addr_eq(self, other.as_ptr())
      || (
      self.symbol() == other.symbol()
          && self.compare_arguments(other) == Ordering::Equal
      )
  }
  
  // endregion Comparison
  
  // region Copy Constructors

  /// For hash consing, recursively checks child nodes to determine if a canonical copy needs to be made.
  fn make_canonical(&self, hash_cons_set: &mut HashConsSet) -> DagNodePtr;

  /// For hash consing unreduced nodes, recursively creates a canonical copy.
  fn make_canonical_copy(&self, hash_cons_set: &mut HashConsSet) -> DagNodePtr;
  
  /// Makes a shallow clone of this node.
  fn make_clone(&self) -> DagNodePtr;

  /// Overwrites other with a clone of self. Invalidates existing fat pointers.
  /// MUST be overridden for nonstandard args or inline data that needs to be cloned.
  fn overwrite_with_clone(&mut self, mut other: DagNodePtr) -> DagNodePtr {
    let node_mut = other.core_mut();

    // Overwrite all `DagNodeCore` fields. 
    node_mut.args       = self.shallow_copy_args();
    node_mut.inline     = self.core().inline;
    node_mut.theory_tag = self.core().theory_tag;
    node_mut.symbol     = self.symbol();
    node_mut.sort_index = self.sort_index();
    // Copy over just the rewriting flags
    let rewrite_flags   = self.flags() & DagNodeFlag::RewritingFlags;
    node_mut.flags      = rewrite_flags;

    DagNodeCore::upgrade(node_mut)
  }

  // endregion Copy Constructors

  // region GC related methods

  /// MUST override if `Self::args` is not a `DagNodeVector`.
  fn mark(&mut self) {
    if self.core().is_marked() {
      return;
    }

    ACTIVE_NODE_COUNT.fetch_add(1, Relaxed);
    self.core_mut().flags.insert(DagNodeFlag::Marked);

    // The empty case
    if self.core().args.is_null() {
      // pass
    } // The vector case
    else if self.core().needs_destruction() {
      {
        // Scope for mutable reference.
        let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);

        for node in node_vector.iter_mut() {
          node.mark();
        }
      }
      // Reallocate
      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);
      self.core_mut().args = (node_vector.copy() as *mut DagNodeVector) as *mut u8;

    } // The singleton case
    else {
      // Guaranteed to be non-null.
      let mut node: DagNodePtr = arg_to_dag_node(self.core().args);
      node.mark();
    }
  } // end fn mark
  
  fn shallow_copy_args(&self) -> *mut u8 {
    if !self.core().args.is_null() && self.core().needs_destruction() {
      // Reallocate
      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);
      (node_vector.copy() as *mut DagNodeVector) as *mut u8
    } // The empty or singleton case
    else {
      // Be careful: 
      self.core().args
    }
  }

  /// Finalize is run when this node is swept during garbage collection if its `NeedsDestruction` flag is set. The
  /// finalizer should only release whatever it is directly responsible for and cannot assume any of its children exist.
  fn finalize(&mut self) {
    /* empty default implementation */
  }

  // endregion GC related methods
}

// region trait impls for DagNode

// ToDo: Revisit whether `semantic_hash` is appropriate for the `Hash` trait.
// Use the `DagNode::compute_hash(…)` hash for `HashSet`s and friends.
impl Hash for dyn DagNode {
  fn hash<H: Hasher>(&self, state: &mut H) {
    state.write_u32(self.structural_hash())
  }
}
// To use `DagNode` with `HashSet`, it needs to implement `Eq`
impl PartialEq for dyn DagNode {
  fn eq(&self, other: &Self) -> bool {
    self.structural_hash() == other.structural_hash()
  }
}
impl Eq for dyn DagNode {}

impl Formattable for dyn DagNode {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    match style {
      FormatStyle::Debug => {
        write!(f, "<{}, {:p}>", self.symbol(), self.as_ptr().as_ptr())?
      }

      _ => {
        write!(f, "<{}>", self.symbol())?
      }
    };

    if self.len() > 0 {
      let mut args = self.iter_args();
      write!(f, "(")?;
      args.next().unwrap().repr(f, style)?;
      for arg in args {
        write!(f, ", ")?;
        arg.repr(f, style)?;
      }
      write!(f, ")")?;
    }

    Ok(())

  }
}

impl_display_debug_for_formattable!(dyn DagNode);

// endregion trait impls for DagNode

// Unsafe private free functions

/// Reinterprets `args` as a `DagNodePtr`. The caller MUST be sure
/// that `args` actually points to a `DagNode`.
#[inline(always)]
pub fn arg_to_dag_node(args: *mut u8) -> DagNodePtr {
  DagNodeCore::upgrade(args as ThinDagNodePtr)
}

/// Reinterprets `args` as a `DagNodeVectorRefMut`. The caller MUST
/// be sure that `args` actually points to a `DagNodeVector`.
#[inline(always)]
pub fn arg_to_node_vec(args: *mut u8) -> DagNodeVectorRefMut {
  unsafe { (args as *mut DagNodeVector).as_mut().unwrap() }
}
