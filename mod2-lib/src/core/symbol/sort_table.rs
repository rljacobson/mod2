use mod2_abs::NatSet;
use crate::api::Arity;
use crate::core::sort::kind::KindPtr;
use crate::core::sort::SortPtr;
use crate::core::symbol::op_declaration::{ConstructorStatus, OpDeclaration};

// ToDo: Most of these vectors are likely to be small. Benchmark with tiny_vec.
#[derive(PartialEq, Eq, Default)]
pub struct SortTable {
  arg_count:                 i16,
  op_declarations:           Vec<OpDeclaration>,
  arg_kinds:                 Vec<KindPtr>,      // "component vector"
  sort_diagram:              Vec<i32>,
  single_non_error_sort:     Option<SortPtr>,   // if we can only generate one non-error sort
  constructor_diagram:       Vec<i32>,
  maximal_op_decl_set_table: Vec<NatSet>,       // indices of maximal op decls with range <= each sort
}

impl SortTable {
  /// Is the symbol strictly a constructor (non constructor)? Used to determine if
  /// every member of the kind can share a single constructor. 
  #[inline(always)]
  pub fn constructor_status(&self) -> ConstructorStatus {
    let mut constructor_status = ConstructorStatus::Unspecified;
    for declaration in &self.op_declarations {
      constructor_status |= declaration.is_constructor;
    }
    constructor_status
  }
  
  #[inline(always)]
  pub fn arity(&self) -> Arity {
    self.arg_count.into()
  }

  #[inline(always)]
  pub fn get_maximal_op_decl_set(&mut self, target: SortPtr) -> &NatSet {
    if self.maximal_op_decl_set_table.is_empty() {
      self.compute_maximal_op_decl_set_table();
    }
    &self.maximal_op_decl_set_table[target.index_within_kind as usize]
  }

  #[inline(always)]
  pub fn special_sort_handling(&self) -> bool {
    self.sort_diagram.is_empty()
  }

  #[inline(always)]
  pub fn add_op_declaration(&mut self, op_declaration: OpDeclaration) {
    if self.op_declarations.is_empty() {
      self.arg_count = op_declaration.arity();
    } else { 
      assert_eq!(
        (op_declaration.len() - 1) as i16,
        self.arg_count,
        "bad domain length of {} instead of {}",
        (op_declaration.len() - 1) as i16,
        self.arg_count
      );
    }

    self.op_declarations.push(op_declaration);
  }

  #[inline(always)]
  pub fn get_op_declarations(&self) -> &Vec<OpDeclaration> {
    &self.op_declarations
  }

  #[inline(always)]
  pub fn range_component(&self) -> KindPtr {
    // ToDo: Is this function fallible? Should it return `Option<KindPtr>`?
    //       If this is only ever called after `Module::compute_kind_closures()`, this is safe.
    unsafe { (&self.op_declarations[0])[self.arg_count as usize].kind.unwrap_unchecked() }
  }

  /// If an operator has been declared with multiple range sort, this
  /// function just returns the first, which is good enough for some
  /// purposes.
  #[inline(always)]
  pub fn get_range_sort(&self) -> SortPtr {
    (&self.op_declarations[0])[self.arg_count as usize]
  }

  #[inline(always)]
  pub fn domain_component(&self, arg_nr: usize) -> KindPtr {
    unsafe { (&self.op_declarations[0])[arg_nr].kind.unwrap_unchecked() }
  }

  // #[inline(always)]
  // pub fn domain_components_iter(&self) -> Box<dyn Iterator<Item = KindPtr>> {
  //   Box::new(
  //     (&self.op_declarations[0])
  //         .iter()
  //         .map(|v| unsafe{ v.kind.unwrap_unchecked() }),
  //   )
  // }

  #[inline(always)]
  pub fn get_single_non_error_sort(&self) -> Option<SortPtr> {
    self.single_non_error_sort.clone()
  }
  
  #[inline(always)]
  pub fn traverse(&self, position: usize, sort_index: usize) -> i32 {
    // ToDo: Do we need a bounds check?
    self.sort_diagram[position + sort_index]
  }

  #[inline(always)]
  pub fn constructor_traverse(&self, position: usize, sort_index: usize) -> i32 {
    // ToDo: Do we need a bounds check?
    self.constructor_diagram[position + sort_index]
  }

  pub fn domain_subsumes(&self, subsumer: usize, victim: usize) -> bool {
    let s = &self.op_declarations[subsumer];
    let v = &self.op_declarations[victim];

    for i in 0..self.arg_count as usize {
      if !v[i].leq(s[i]) {
        return false;
      }
    }
    true
  }

  pub fn compute_maximal_op_decl_set_table(&mut self) {
    let range             = self.range_component();
    let sort_count        = range.sort_count();
    let declaration_count = self.op_declarations.len();

    self.maximal_op_decl_set_table
        .resize(sort_count, NatSet::new());

    for i in 0..sort_count {
      let target = range.sort(i);

      for j in 0..declaration_count {
        if (&self.op_declarations[j])[self.arg_count as usize].leq(target) {
          for k in 0..j {
            if self.maximal_op_decl_set_table[i].contains(k) {
              if self.domain_subsumes(k, j) {
                continue;
              } else if self.domain_subsumes(j, k) {
                self.maximal_op_decl_set_table[i].remove(k);
              }
            }
          }

          self.maximal_op_decl_set_table[i].insert(j);
        }
      }
    }
  }
  
  /// The sort diagram is a data structure used to efficiently determine the result sort
  /// of an operator application in Maude, based on the sorts of its arguments. It encodes
  /// all valid combinations of argument sorts and their corresponding result sorts,
  /// allowing for fast, table-driven lookup at runtime. This eliminates the need to search
  /// through all operator declarations, supports polymorphic operators, and ensures that
  /// sort constraints are enforced consistently and efficiently during term rewriting.
  fn build_sort_diagram(&mut self) {
    todo!()
  }
}
