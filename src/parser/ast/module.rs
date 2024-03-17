use std::collections::HashMap;
use crate::abstractions::IString;
use crate::core::module::Module;
use crate::core::pre_equation::{PreEquation, PreEquationKind};
use crate::core::pre_equation::condition::Conditions;
use crate::core::sort::collection::SortCollection;
use crate::parser::ast::{BxEquationDeclarationAST, BxRuleDeclarationAST, BxSortDeclarationAST, ItemAST, symbol_decl};
use crate::parser::ast::symbol_decl::{BxSymbolDeclarationAST, BxVariableDeclarationAST};
use crate::theory::symbol::RcSymbol;
use crate::theory::symbol_type::CoreSymbolType;

pub(crate) type BxModuleAST = Box<ModuleAST>;

/// The `Module` AST is the top level AST node.
pub(crate) struct ModuleAST {
  pub name : IString,
  pub items: Vec<ItemAST>
}

impl ModuleAST {

  /// Constructs a `Module` representation of `self`, consuming `self`.
  pub fn construct_module(mut self) -> Module {
    // The items of the module are binned according to type before processing.
    let mut modules   : Vec<BxModuleAST>              = Vec::new();
    let mut var_decls : Vec<BxVariableDeclarationAST> = Vec::new();
    let mut sym_decls : Vec<BxSymbolDeclarationAST>   = Vec::new();
    let mut sort_decls: Vec<BxSortDeclarationAST>     = Vec::new();
    let mut rule_decls: Vec<BxRuleDeclarationAST>     = Vec::new();
    let mut eq_decls  : Vec<BxEquationDeclarationAST> = Vec::new();

    for item in self.items.drain(..) {
      match item {
        ItemAST::Submodule(i) => modules.push(i),
        ItemAST::VarDecl(i)   => var_decls.push(i),
        ItemAST::SymDecl(i)   => sym_decls.push(i),
        ItemAST::SortDecl(i)  => sort_decls.push(i),
        ItemAST::Rule(i)      => rule_decls.push(i),
        ItemAST::Equation(i)  => eq_decls.push(i),
      }
    }

    // Todo: How submodules work determines how we construct modules. See [Design Questions](doc/DesignQuestions.md).
    /*
    Sorts can be declared explicitly, or they can be implicitly declared by being referenced without declaration.
    The transitive closure of the subsort relation and the construction of the connected components is done in
    `Module::close_sort_set()` method.

    Every sort that is encountered is checked to see if it has already been created. If it has, the existing sort
    object is fetched. Otherwise, the sort is created.
    */
    let mut sorts  : SortCollection             = SortCollection::new();
    let mut symbols: HashMap<IString, RcSymbol> = HashMap::new();

    // Sort Declarations
    for sort_decl in sort_decls.iter() {
      for subsort_name in sort_decl.sorts_lt.iter() {
        // Get or insert new subsort.
        let subsort = sorts.get_or_create_sort(*subsort_name);
        for supersort_name in sort_decl.sorts_gt.iter() {
          assert_ne!(*subsort_name, *supersort_name, "sort declared as a subsort of itself");

          // Get or insert new supersort.
          let supersort = sorts.get_or_create_sort(*supersort_name);
          // ToDo: Check that this constraint has not already been declared by checking that `subsort.supersorts` does
          //       not already contain `supersort` (and vice versa).
          subsort.borrow_mut().supersorts.push(supersort.downgrade());
          supersort.borrow_mut().subsorts.push(subsort.downgrade());
        }
      }
    }

    // Variable Declarations
    for var_decl in var_decls {
      symbol_decl::construct_symbol_from_decl(
        &mut symbols,
        &mut sorts,
        var_decl.name,
        var_decl.sort_spec,
        var_decl.arity,
        var_decl.attributes,
        CoreSymbolType::Variable
      );
    }

    // Symbol Declarations
    for sym_decl in sym_decls {
      symbol_decl::construct_symbol_from_decl(
        &mut symbols,
        &mut sorts,
        sym_decl.name,
        sym_decl.sort_spec,
        sym_decl.arity,
        sym_decl.attributes,
        CoreSymbolType::Standard
      );
    }

    let mut pre_equations: Vec<PreEquation> = Vec::new();

    // Rule Declarations
    for rule_decl in rule_decls {
      let lhs  = rule_decl.lhs.construct(&mut symbols);
      let rhs  = rule_decl.rhs.construct(&mut symbols);
      let rule = PreEquationKind::Rule{
        rhs_term: Box::new(rhs),
      };
      let conditions: Conditions
          = rule_decl.conditions
                     .unwrap_or_default()
                     .into_iter()
                     .map(|c| Box::new(c.construct(&mut symbols, &mut sorts)))
                     .collect();

      let pre_equation = PreEquation{
        name      : None,
        attributes: Default::default(),
        conditions,
        lhs_term  : Box::new(lhs),
        kind      : rule,
      };

      pre_equations.push(pre_equation);
    }

    // Equation Declarations
    for eq_decl in eq_decls {
      let lhs  = eq_decl.lhs.construct(&mut symbols);
      let rhs  = eq_decl.rhs.construct(&mut symbols);
      let rule = PreEquationKind::Equation{
        rhs_term: Box::new(rhs),
      };
      let conditions: Conditions
          = eq_decl.conditions
                   .unwrap_or_default()
                   .into_iter()
                   .map(|c| Box::new(c.construct(&mut symbols, &mut sorts)))
                   .collect();

      let pre_equation = PreEquation{
        name      : None,
        attributes: Default::default(),
        conditions,
        lhs_term  : Box::new(lhs),
        kind      : rule,
      };

      pre_equations.push(pre_equation);
    }

    Module{
      name      : Default::default(),
      submodules: vec![],
      status    : Default::default(),
      sorts,
      kinds     : vec![],
      symbols,
    }
  }
}
