/*!

The `DagNode` trait is the interface all DAG node's must implement.

Requirements of implementers of `DagNode`:
 1. DAG nodes should be newtypes of `DagNodeCore`. In particular...
 2. DAG nodes *must* have the same memory representation as a `DagNodeCore`.
 3. Implementers of `DagNode` are responsible for casting pointers, in particular its arguments.

*/

use std::{
  any::Any,
  cmp::{max, Ordering},
  fmt::{Display, Formatter},
  iter::Iterator,
  ops::Deref,
  rc::Rc,
};
use std::ops::DerefMut;
use mod2_abs::UnsafePtr;
use crate::{api::{
  Arity,
  symbol::SymbolPtr
}, core::{
  gc::{
    gc_vector::{GCVector, GCVectorRefMut},
    increment_active_node_count
  },
  dag_node_core::{
    DagNodeCore,
    DagNodeFlag,
    DagNodeFlags,
    ThinDagNodePtr
  },
  sort::{SortPtr, SpecialSort},
  symbol_core::SymbolCore,
  format::{FormatStyle, Formattable}
}, impl_display_debug_for_formattable};

// A fat pointer to a trait object. For a thin pointer to a DagNodeCore, use ThinDagNodePtr
pub type DagNodePtr    = UnsafePtr<dyn DagNode + 'static>;
pub type DagNodeVector = GCVector<DagNodePtr>;
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

  #[inline(always)]
  fn as_ptr_mut(&self) -> *mut dyn DagNode where Self: Sized {
    let ptr: *const dyn DagNode = self;
    ptr as *mut dyn DagNode
  }


  // region Accessors

  /// Trait level access to members for shared implementation
  fn core(&self) -> &DagNodeCore;
  fn core_mut(&mut self) -> &mut DagNodeCore;

  #[inline(always)]
  fn arity(&self) -> Arity {
    self.core().arity()
  }


  /// MUST override if Self::args is not a `DagNodeVector`
  fn iter_args(&self) -> Box<dyn Iterator<Item=DagNodePtr>> {
    // For assertions
    // ToDo: These assertions will need to change for variadic nodes.
    let arity = if let Arity::Value(v) = self.arity() { v } else { 0 };

    // The empty case
    if self.core().args.is_null() {
      assert_eq!(arity, 0);
      Box::new(std::iter::empty())
    } // The vector case
    else if self.core().needs_destruction() {
      assert!(arity>1);

      let node_vector: DagNodeVectorRefMut = arg_to_node_vec(self.core().args);
      Box::new(node_vector.iter().cloned())
    } // The singleton case
    else {
      assert_eq!(arity, 1);

      let node = arg_to_dag_node(self.core().args);

      // Make a fat pointer to the single node and return an iterator to it. This allows `self` to
      // escape the method. Of course, `self` actually points to a `DagNode` that is valid for the
      // lifetime of the program, so even in the event of the GC equivalent of a dangling pointer
      // or use after free, this will be safe. (Strictly speaking, it's probably UB.)
      let v = unsafe { std::slice::from_raw_parts(&node, 1) };
      Box::new(v.iter().map(|n| *n))
    }
  }

  /// MUST override if Self::args is not a `DagNodeVector`
  fn insert_child(&mut self, new_child: DagNodePtr){
    // ToDo: Should we signal if arity is exceeded and/or DagNodeVector needs to reallocate?

    // Empty case
    if self.core().args.is_null() {
      self.core_mut().args = new_child.as_mut_ptr() as *mut u8;
    } // Vector case
    else if self.core().needs_destruction() {
      let node_vec: DagNodeVectorRefMut = arg_to_node_vec(self.core_mut().args);
      node_vec.push(new_child)
    } // Singleton case
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

  fn equals(&self, other: DagNodePtr) -> bool {
    std::ptr::addr_eq(self, other.as_ptr())
      || (
      self.symbol() == other.symbol()
          && self.compare_arguments(other) == Ordering::Equal
      )
  }

  // endregion

  // region GC related methods

  /// MUST override if `Self::args` is not a `DagNodeVector`.
  fn mark(&mut self) {
    if self.core().is_marked() {
      return;
    }

    increment_active_node_count();
    self.core_mut().flags.insert(DagNodeFlag::Marked);

    // The empty case
    if self.core().args.is_null() {
      // pass
    } // The vector case
    else if self.core().needs_destruction() {
      {
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

  // endregion GC related methods
}

impl Formattable for dyn DagNode {
  fn repr(&self, f: &mut dyn std::fmt::Write, style: FormatStyle) -> std::fmt::Result {
    match style {
      FormatStyle::Debug => {
        write!(f, "DagNode<{}>", self.symbol())
      }
      
      _ => {
        write!(f, "<{}>", self.symbol())
      }
    }
    
  }
}

impl_display_debug_for_formattable!(dyn DagNode);



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
  unsafe { (args as *mut DagNodeVector).as_mut_unchecked() }
}
