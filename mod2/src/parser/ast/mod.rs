/*!

Defines the AST data structures for the language described by the EBNF grammar below. The `Module` AST node is the
top-most node.

The `Module` node's `Module::construct(..)` method implements code to convert the AST into the final internal representation. It calls other `construct` methods on other node types.
<br><br>

## EBNF Grammar

```ebnf
Identifier := [a-zA-Z][a-zA-Z_]* ;
AndOp      := r"/\" | "∧" | "⋀" ;
OrOp       := r"\/" | "∨" | "⋁" ;
ArrowOp    := "->" ;
SortOp  := "::" ;
RuleOp     := "=>" ;
EqualOp    := "="  ;
MatchOp    := ":=" ;

# Syntactic Rules

Module := Item* ;

Item := Declaration
      | Submodule
      # | Statement
      ;

Declaration := VariableDeclaration
            | SymbolDeclaration
            | SortDeclaration
            | RuleDeclaration
            | EquationDeclaration
            # | StrategyDeclaration
            ;

SortDeclaration := "sort" SortList ("<" SortList)? ";" ;

SortList := Identifier ("," Identifier)* ;

SymbolDeclaration := ("symbol"|"sym") Identifier ("/" NaturalNumber)? (SortOp SortSpec)? ConditionSpec? AttributeSpec? ";" ;

VariableDeclaration := ("variable"|"var") Identifier ("/" NaturalNumber)? (SortOp SortSpec)? ConditionSpec? AttributeSpec? ";" ;

#Operator := ("operator"|"op") Identifier ("/" NaturalNumber)? (SortOp SortSpec)? ConditionSpec? AttributeSpec? ";" ;

RuleDeclaration := ("rule" | "rl") Term RuleOp Term ConditionSpec? ";" ;

EquationDeclaration := ("equation" | "eq") Term EqualOp Term ConditionSpec? ";" ;

MembershipDeclaration := ("membership" | "mb") Term SortOp SortSpec ConditionSpec? ";" ;

Submodule := "mod" Identifier "{" Module "}" ;

# Statement := BindStatement
#            | ReduceStatement
#            | MatchStatement
#            | MatchAllStatement
#            | UnifyStatement
#            | ReplaceStatement
#            | ReplaceAllStatement
#            ;

Term :=
    Identifier
    | Term "(" Term ("," Term)* ")"
    | "(" Term ")"
    ;

SortSpec :=
    Identifier
    | SortSpec+ ArrowOp SortSpec
    | "(" SortSpec ")"
    ;

Attribute :=
    "assoc" | "associative"
    | "comm" | "commutative"
    | "ctor" | "constructor"
    | "id" "(" Term ")"
    ;

AttributeSpec := "[" AttributeList "]" ;

ConditionSpec := "if" Condition (AndOp Condition)* ;

Condition :=
    EqualityCondition
    | MatchCondition
    | MembershipCondition
    | RewriteCondition
    ;

EqualityCondition :=
    Term EqualOp Term # Regular Equation
    | Term            # Boolean Expression (short for term = true)
    ;

MatchCondition := Term MatchOp Term ;

MembershipCondition := Term SortOp SortSpec ;

RewriteCondition := Term RuleOp Term ;

```

(The `#` symbol begins a comment, disabling those lines it comments out.)

*/

use std::collections::hash_map::Entry;
use mod2_abs::{heap_construct, HashMap, IString, SmallVec};
use mod2_lib::{
  api::{
    built_in::get_built_in_symbol,
    free_theory::FreeSymbol,
    Symbol,
    SymbolPtr,
  },
  core::sort::SortPtr
};

mod attribute;
mod condition;
mod module;
mod sort;
mod symbol_decl;
mod term;

pub use attribute::*;
pub use condition::*;
pub use module::*;
pub use sort::*;
pub use symbol_decl::*;
pub use term::*;


/// An item is anything that lives in a module.
pub(crate) enum ItemAST {
  Submodule(BxModuleAST),
  VarDecl(BxVariableDeclarationAST),
  SymDecl(BxSymbolDeclarationAST),
  SortDecl(BxSortDeclarationAST),
  Rule(BxRuleDeclarationAST),
  Equation(BxEquationDeclarationAST),
  Membership(BxMembershipDeclarationAST)
}

/// A sort declaration has the form
///     SortDeclaration := "sort" SortList ("<" SortList)? ";" ;
/// Not to be confused with membership axioms introduced with the `membership` keyword.
pub(crate) type BxSortDeclarationAST = Box<SortDeclarationAST>;
pub(crate) struct SortDeclarationAST {
  pub sorts_lt: Vec<IString>,
  pub sorts_gt: Vec<IString>,
}

/// Declaration of the form
///     RuleDeclaration := ("rule" | "rl") Term RuleOp Term ConditionSpec? ";" ;
pub(crate) type BxRuleDeclarationAST = Box<RuleDeclarationAST>;
pub(crate) struct RuleDeclarationAST {
  pub lhs       : BxTermAST,
  pub rhs       : BxTermAST,
  pub conditions: Option<Vec<ConditionAST>>
}

/// Declaration of the form
///     EquationDeclaration := ("equation" | "eq") Term EqualOp Term ConditionSpec? ";" ;
pub(crate) type BxEquationDeclarationAST = Box<EquationDeclarationAST>;
pub(crate) struct EquationDeclarationAST {
  pub lhs       : BxTermAST,
  pub rhs       : BxTermAST,
  pub conditions: Option<Vec<ConditionAST>>
}


/// Declaration of the form
///     MembershipDeclaration := ("membership" | "mb") Term SortOp SortSpec ConditionSpec? ";" ;
pub(crate) type BxMembershipDeclarationAST = Box<MembershipDeclarationAST>;
pub(crate) struct MembershipDeclarationAST {
  pub lhs       : BxTermAST,
  pub rhs       : BxSortIdAST,
  pub conditions: Option<Vec<ConditionAST>>
}

/// This method is intended for implicitly defined symbols, that is, symbols that are used without being declared.
/// By default, we make such symbols free symbols.
fn get_or_create_symbol<T:Into<IString>>(name: T, symbols: &mut HashMap<IString, SymbolPtr>) -> SymbolPtr {
  let name = name.into();

  // Built-ins like `true` and `false`
  // ToDo: If the user tries to shadow a built-in, usages will still reference the built-in. This may not be the right
  //       semantics once shadowing built-ins is made a warning/error.
  if let Some(symbol) = get_built_in_symbol(&*name) {
    return symbol;
  }

  match symbols.entry(name.clone()) {
    Entry::Occupied(s) => *s.get(),
    Entry::Vacant(v) => {
      let s = heap_construct!(FreeSymbol::with_arity(name.clone(), 0.into(), None));
      let s = SymbolPtr::new(s);
      v.insert(s);
      s
    }
  }
}
