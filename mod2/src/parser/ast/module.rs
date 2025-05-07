use std::{
  collections::HashMap,
  fmt::{Debug, Formatter}
};

use mod2_abs::IString;

use mod2_lib::{
  api::{
    symbol::{
      Symbol,
      SymbolPtr
    },
    term::BxTerm,
    variable_theory::VariableTerm,
  },
  core::{
    BxModule, 
    Module,
    pre_equation::{
      PreEquation,
      PreEquationKind,
      condition::Conditions
    },
    sort::collection::SortCollection,
    symbol::SymbolType
  },
};

use crate::{
  parser::ast::{
    BxEquationDeclarationAST,
    BxMembershipDeclarationAST,
    BxRuleDeclarationAST,
    BxSortDeclarationAST,
    ItemAST,
    construct_symbol_term_from_decl,
    symbol_decl::{
      BxSymbolDeclarationAST,
      BxVariableDeclarationAST
    }
  },
};

pub(crate) type BxModuleAST = Box<ModuleAST>;

/// The `Module` AST is the top level AST node.
pub(crate) struct ModuleAST {
  pub name : IString,
  pub items: Vec<ItemAST>
}

impl ModuleAST {

  /// Constructs a `Module` representation of `self`, consuming `self`.
  // ToDo: Inherit parent's environment (symbols/sorts/variables)
  pub fn construct_module(mut self) -> BxModule {
    // The items of the module are binned according to type before processing.
    let mut modules   : Vec<BxModuleAST>                = Vec::new();
    let mut var_decls : Vec<BxVariableDeclarationAST>   = Vec::new();
    let mut sym_decls : Vec<BxSymbolDeclarationAST>     = Vec::new();
    let mut sort_decls: Vec<BxSortDeclarationAST>       = Vec::new();
    let mut rule_decls: Vec<BxRuleDeclarationAST>       = Vec::new();
    let mut eq_decls  : Vec<BxEquationDeclarationAST>   = Vec::new();
    let mut mb_decls  : Vec<BxMembershipDeclarationAST> = Vec::new();

    for item in self.items.drain(..) {
      match item {
        ItemAST::Submodule(i)  => modules.push(i),
        ItemAST::VarDecl(i)    => var_decls.push(i),
        ItemAST::SymDecl(i)    => sym_decls.push(i),
        ItemAST::SortDecl(i)   => sort_decls.push(i),
        ItemAST::Rule(i)       => rule_decls.push(i),
        ItemAST::Equation(i)   => eq_decls.push(i),
        ItemAST::Membership(i) => mb_decls.push(i)
      }
    }

    // Todo: How submodules work determines how we construct modules. See [the Design doc](doc/DesignNotes.md).
    /*
    Sorts can be declared explicitly, or they can be implicitly declared by being referenced without declaration.
    The transitive closure of the subsort relation and the construction of the connected components is done in
    `Module::close_sort_set()` method.

    Every sort that is encountered is checked to see if it has already been created. If it has, the existing sort
    object is fetched. Otherwise, the sort is created.
    */
    let mut sorts  : SortCollection              = SortCollection::new();
    let mut symbols: HashMap<IString, SymbolPtr> = HashMap::new();

    // Sort Declarations
    for sort_decl in sort_decls.iter() {
      for subsort_name in sort_decl.sorts_lt.iter() {
        // Get or insert new subsort.
        let mut subsort = sorts.get_or_create_sort(subsort_name.clone());
        for supersort_name in sort_decl.sorts_gt.iter() {
          assert_ne!(*subsort_name, *supersort_name, "sort declared as a subsort of itself");

          // Get or insert new supersort.
          let mut supersort = sorts.get_or_create_sort(supersort_name.clone());
          // ToDo: Check that this constraint has not already been declared by checking that `subsort.supersorts` does
          //       not already contain `supersort` (and vice versa).
          unsafe {
            subsort.supersorts.push(supersort);
            supersort.subsorts.push(subsort);
          }
        }
      }
    }

    // Variable Declarations
    let mut variables: HashMap<IString, BxTerm> = HashMap::with_capacity(var_decls.len());
    for var_decl in var_decls {
      let var  = var_decl.construct(&mut symbols, &mut sorts);
      let name = var.as_any().downcast_ref::<VariableTerm>().unwrap().name.clone();
      variables.insert(name, var);
    }

    // Symbol Declarations
    for sym_decl in sym_decls {
      sym_decl.construct(&mut symbols, &mut sorts);
    }


    // Rule Declarations
    let mut rules: Vec<PreEquation> = Vec::with_capacity(rule_decls.len());
    for rule_decl in rule_decls {
      let lhs  = rule_decl.lhs.construct(&mut symbols, &mut sorts, &mut variables);
      let rhs  = rule_decl.rhs.construct(&mut symbols, &mut sorts, &mut variables);
      let rule = PreEquationKind::Rule{
        rhs_term: rhs,
      };
      let conditions: Conditions
          = rule_decl.conditions
                     .unwrap_or_default()
                     .into_iter()
                     .map(|c| Box::new(c.construct(&mut symbols, &mut sorts, &mut variables)))
                     .collect();

      let pre_equation = PreEquation{
        name      : None,
        attributes: Default::default(),
        conditions,
        lhs_term  : lhs,
        pe_kind   : rule,
      };

      rules.push(pre_equation);
    }


    // Equation Declarations
    let mut equations: Vec<PreEquation> = Vec::with_capacity(eq_decls.len());
    for eq_decl in eq_decls {
      let lhs      = eq_decl.lhs.construct(&mut symbols, &mut sorts, &mut variables);
      let rhs      = eq_decl.rhs.construct(&mut symbols, &mut sorts, &mut variables);
      let equation = PreEquationKind::Equation{
        rhs_term: rhs,
      };
      let conditions: Conditions
          = eq_decl.conditions
                   .unwrap_or_default()
                   .into_iter()
                   .map(|c| Box::new(c.construct(&mut symbols, &mut sorts, &mut variables)))
                   .collect();

      let pre_equation = PreEquation{
        name      : None,
        attributes: Default::default(),
        conditions,
        lhs_term  : lhs,
        pe_kind   : equation,
      };

      equations.push(pre_equation);
    }


    // Membership Axiom Declarations
    let mut membership: Vec<PreEquation> = Vec::with_capacity(mb_decls.len());
    for mb_decl in mb_decls {
      let lhs        = mb_decl.lhs.construct(&mut symbols, &mut sorts, &mut variables);
      let rhs        = mb_decl.rhs.construct(&mut sorts);
      let membership = PreEquationKind::Membership{
        sort: rhs,
      };
      let conditions: Conditions
          = mb_decl.conditions
                   .unwrap_or_default()
                   .into_iter()
                   .map(|c| Box::new(c.construct(&mut symbols, &mut sorts, &mut variables)))
                   .collect();

      let pre_equation = PreEquation{
        name      : None,
        attributes: Default::default(),
        conditions,
        lhs_term  : lhs,
        pe_kind   : membership,
      };

      equations.push(pre_equation);
    }


    // Submodules
    let mut submodules: Vec<BxModule> =
        modules.into_iter()
               .map(|m| m.construct_module())
               .collect();


    let mut new_module = Module{
      name      : self.name,
      status    : Default::default(),
      kinds     : vec![], // computed below
      submodules,
      sorts,
      symbols,
      rules,
      equations,
      membership,
      variables,
    };
    unsafe {
      new_module.compute_kind_closures();
    }
    
    Box::new(new_module)
  }
}

// Todo: Implement a more appropriate debug representation.
impl Debug for ModuleAST {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "<ModuleAST>")
  }
}
