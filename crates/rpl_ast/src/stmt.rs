//! Statement, declaration, pattern matching, and block structures.

use std::ops::{Deref, DerefMut};

use crate::expr::Expr;
use crate::literal::Literal;
use crate::span::Span;
use crate::types::Type;

/// Function declaration parameter with name and type annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// Parameter identifier name.
    pub name: String,
    /// Parameter type annotation.
    pub param_type: Type,
    /// Source code span of the parameter declaration.
    pub span: Span,
}

impl Param {
    /// Creates a new function parameter.
    pub const fn new(name: String, param_type: Type, span: Span) -> Self {
        Self {
            name,
            param_type,
            span,
        }
    }
}

/// Composite type field with name and type annotation.
#[derive(Debug, Clone, PartialEq)]
pub struct Field {
    /// Field identifier name.
    pub name: String,
    /// Field type annotation.
    pub field_type: Type,
    /// Source code span of the field declaration.
    pub span: Span,
}

impl Field {
    /// Creates a new type declaration field.
    pub const fn new(name: String, field_type: Type, span: Span) -> Self {
        Self {
            name,
            field_type,
            span,
        }
    }
}

/// Pattern matching target patterns in `match` statements.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// Literal constant pattern (`true`, `false`, `unknown`, `42`, `"text"`).
    Literal(Literal, Span),
    /// Variable binding pattern (`x`).
    Identifier(String, Span),
    /// Constructor pattern (`Ok(v)`, `Error(e)`, `Some(x)`).
    Constructor {
        /// Constructor name.
        name: String,
        /// Positional argument patterns.
        args: Vec<Pattern>,
        /// Source code span of the constructor pattern.
        span: Span,
    },
    /// Wildcard pattern (`_`).
    Wildcard(Span),
}

impl Pattern {
    /// Returns the source code span of this pattern.
    pub fn span(&self) -> Span {
        match self {
            Self::Literal(_, span) => *span,
            Self::Identifier(_, span) => *span,
            Self::Constructor { span, .. } => *span,
            Self::Wildcard(span) => *span,
        }
    }
}

/// A single case branch within a `match` statement.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchCase {
    /// Pattern matched against the subject expression.
    pub pattern: Pattern,
    /// Optional match guard expression (`case pattern if guard:`).
    pub guard: Option<Expr>,
    /// Block executed when the pattern matches.
    pub body: Block,
    /// Source code span of this case.
    pub span: Span,
}

impl MatchCase {
    /// Creates a new match case without a guard expression.
    pub const fn new(pattern: Pattern, body: Block, span: Span) -> Self {
        Self {
            pattern,
            guard: None,
            body,
            span,
        }
    }

    /// Creates a new match case with a guard expression.
    pub const fn with_guard(pattern: Pattern, guard: Expr, body: Block, span: Span) -> Self {
        Self {
            pattern,
            guard: Some(guard),
            body,
            span,
        }
    }
}

/// A block containing a sequence of statements delimited by `:` and `end`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Block {
    /// Ordered list of statements within the block.
    pub stmts: Vec<Stmt>,
    /// Source code span of the entire block.
    pub span: Span,
}

impl Block {
    /// Creates a new block from statements and an explicit span.
    pub const fn new(stmts: Vec<Stmt>, span: Span) -> Self {
        Self { stmts, span }
    }

    /// Creates an empty block with a default span.
    pub const fn empty() -> Self {
        Self {
            stmts: Vec::new(),
            span: Span::dummy(),
        }
    }
}

impl Deref for Block {
    type Target = [Stmt];

    fn deref(&self) -> &Self::Target {
        &self.stmts
    }
}

impl DerefMut for Block {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stmts
    }
}

impl From<Vec<Stmt>> for Block {
    fn from(stmts: Vec<Stmt>) -> Self {
        let span = if let (Some(first), Some(last)) = (stmts.first(), stmts.last()) {
            first.span().combine(last.span())
        } else {
            Span::default()
        };
        Self { stmts, span }
    }
}

impl IntoIterator for Block {
    type Item = Stmt;
    type IntoIter = std::vec::IntoIter<Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.stmts.into_iter()
    }
}

impl<'a> IntoIterator for &'a Block {
    type Item = &'a Stmt;
    type IntoIter = std::slice::Iter<'a, Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.stmts.iter()
    }
}

impl<'a> IntoIterator for &'a mut Block {
    type Item = &'a mut Stmt;
    type IntoIter = std::slice::IterMut<'a, Stmt>;

    fn into_iter(self) -> Self::IntoIter {
        self.stmts.iter_mut()
    }
}

/// Statement and declaration AST nodes in RPL.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Immutable variable binding (`let name [: type] = value`).
    Let {
        /// Variable identifier name.
        name: String,
        /// Optional explicit type annotation.
        type_annot: Option<Type>,
        /// Initialization expression.
        value: Expr,
        /// Source code span of the let statement.
        span: Span,
    },

    /// Mutable variable binding (`let mut name [: type] = value`).
    MutLet {
        /// Variable identifier name.
        name: String,
        /// Optional explicit type annotation.
        type_annot: Option<Type>,
        /// Initialization expression.
        value: Expr,
        /// Source code span of the mut let statement.
        span: Span,
    },

    /// Assignment to an existing variable or member (`target = value`).
    Assign {
        /// Target expression (e.g. identifier, member access).
        target: Expr,
        /// Assigned value expression.
        value: Expr,
        /// Source code span of the assignment.
        span: Span,
    },

    /// Function declaration (`fn name(params) [-> return_type]: body end`).
    FnDecl {
        /// Function identifier name.
        name: String,
        /// Formal parameter list.
        params: Vec<Param>,
        /// Optional return type annotation.
        return_type: Option<Type>,
        /// Function body block.
        body: Block,
        /// Source code span of the function declaration.
        span: Span,
    },

    /// Custom composite type definition (`type Name: fields end`).
    TypeDecl {
        /// Type name identifier.
        name: String,
        /// Struct fields.
        fields: Vec<Field>,
        /// Source code span of the type declaration.
        span: Span,
    },

    /// Conditional branch (`if condition: then_branch [else: else_branch] end`).
    If {
        /// Condition expression.
        condition: Expr,
        /// Block executed when condition is true.
        then_branch: Block,
        /// Optional block executed when condition is false.
        else_branch: Option<Block>,
        /// Source code span of the if statement.
        span: Span,
    },

    /// Pattern match statement (`match subject: cases end`).
    Match {
        /// Subject expression being matched against.
        subject: Expr,
        /// List of match case branches.
        cases: Vec<MatchCase>,
        /// Source code span of the match statement.
        span: Span,
    },

    /// Sequential for loop (`for item_name in iterator: body end`).
    For {
        /// Loop variable identifier name.
        item_name: String,
        /// Iterable expression.
        iterator: Expr,
        /// Loop body block.
        body: Block,
        /// Source code span of the for loop.
        span: Span,
    },

    /// Parallel for loop distributing work across worker threads (`parallel for item_name in iterator: body end`).
    ParallelFor {
        /// Loop variable identifier name.
        item_name: String,
        /// Iterable expression.
        iterator: Expr,
        /// Loop body block.
        body: Block,
        /// Source code span of the parallel for loop.
        span: Span,
    },

    /// Asynchronous task spawn (`spawn: body end`).
    Spawn {
        /// Spawned task body block.
        body: Block,
        /// Source code span of the spawn statement.
        span: Span,
    },

    /// Function return statement (`return [expr]`).
    Return(Option<Expr>, Span),

    /// Standalone expression evaluated for side-effects.
    Expr(Expr),
}

impl Stmt {
    /// Returns the source code span of this statement.
    pub fn span(&self) -> Span {
        match self {
            Self::Let { span, .. } => *span,
            Self::MutLet { span, .. } => *span,
            Self::Assign { span, .. } => *span,
            Self::FnDecl { span, .. } => *span,
            Self::TypeDecl { span, .. } => *span,
            Self::If { span, .. } => *span,
            Self::Match { span, .. } => *span,
            Self::For { span, .. } => *span,
            Self::ParallelFor { span, .. } => *span,
            Self::Spawn { span, .. } => *span,
            Self::Return(_, span) => *span,
            Self::Expr(expr) => expr.span(),
        }
    }
}

/// Top-level RPL program or source file AST.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Program {
    /// Top-level statements and declarations.
    pub statements: Vec<Stmt>,
    /// Source code span of the entire program.
    pub span: Span,
}

impl Program {
    /// Creates a new program AST.
    pub const fn new(statements: Vec<Stmt>, span: Span) -> Self {
        Self { statements, span }
    }
}
