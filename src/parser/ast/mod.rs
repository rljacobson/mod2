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


use crate::abstractions::IString;

mod module;
mod term;
mod sort_spec;
mod attribute;
mod condition;
mod symbol_decl;

pub use module::*;
pub use sort_spec::*;
pub use term::*;
pub use attribute::*;
pub use condition::*;
pub use symbol_decl::*;

/// An item is anything that lives in a module.
pub(crate) enum ItemAST {
  Submodule(BxModuleAST),
  VarDecl(BxVariableDeclarationAST),
  SymDecl(BxSymbolDeclarationAST),
  SortDecl(BxSortDeclarationAST),
  Rule(BxRuleDeclarationAST),
  Equation(BxEquationDeclarationAST),
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
