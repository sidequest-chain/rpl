//! Recursive expression tree data structures.

use crate::literal::Literal;
use crate::op::{BinaryOp, UnaryOp};
use crate::span::Span;

/// A fragment within a string interpolation expression (`"Hello $name, $(1 + 2)"`).
#[derive(Debug, Clone, PartialEq)]
pub enum InterpolationFragment {
    /// Static string segment.
    Literal(String),
    /// Dynamic interpolated expression (`$ident` or `$(expr)`).
    Expr(Expr),
}

/// Recursive expression AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Literal constant value with span.
    Literal(Literal, Span),

    /// Identifier reference with span.
    Identifier(String, Span),

    /// Binary operator expression (`left op right`).
    Binary {
        /// Left-hand side operand.
        left: Box<Expr>,
        /// Binary operator.
        op: BinaryOp,
        /// Right-hand side operand.
        right: Box<Expr>,
        /// Source code span of the entire binary expression.
        span: Span,
    },

    /// Unary operator expression (`op expr`).
    Unary {
        /// Unary operator.
        op: UnaryOp,
        /// Sub-expression being operated on.
        expr: Box<Expr>,
        /// Source code span of the unary expression.
        span: Span,
    },

    /// Function call or constructor invocation (`callee(args)`).
    Call {
        /// Expression evaluating to the callable target.
        callee: Box<Expr>,
        /// Positional argument expressions.
        args: Vec<Expr>,
        /// Source code span of the call expression.
        span: Span,
    },

    /// Pipeline expression (`left |> right`).
    Pipe {
        /// Expression yielding the value to pass into the pipe.
        left: Box<Expr>,
        /// Callable target receiving the piped value.
        right: Box<Expr>,
        /// Source code span of the pipe expression.
        span: Span,
    },

    /// Member field or method access (`target.field`).
    MemberAccess {
        /// Base target expression.
        target: Box<Expr>,
        /// Name of the accessed member or field.
        field: String,
        /// Source code span of the member access.
        span: Span,
    },

    /// String interpolation containing text and evaluated sub-expressions.
    StringInterpolation {
        /// Sequence of static text fragments and interpolated expressions.
        fragments: Vec<InterpolationFragment>,
        /// Source code span of the string interpolation expression.
        span: Span,
    },
}

impl Expr {
    /// Returns the source code span of this expression.
    pub fn span(&self) -> Span {
        match self {
            Self::Literal(_, span) => *span,
            Self::Identifier(_, span) => *span,
            Self::Binary { span, .. } => *span,
            Self::Unary { span, .. } => *span,
            Self::Call { span, .. } => *span,
            Self::Pipe { span, .. } => *span,
            Self::MemberAccess { span, .. } => *span,
            Self::StringInterpolation { span, .. } => *span,
        }
    }
}
