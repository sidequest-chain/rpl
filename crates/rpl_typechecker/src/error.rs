//! Diagnostics and error definitions for the RPL typechecker.

use rpl_ast::{BinaryOp, Span, Type, UnaryOp};
use thiserror::Error;

/// Type-checking and semantic analysis errors with source spans.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TypeError {
    /// Type mismatch between expected and actual types.
    #[error("Type mismatch: expected '{expected}', found '{found}'")]
    TypeMismatch {
        expected: Type,
        found: Type,
        span: Span,
    },

    /// Reference to an undefined variable.
    #[error("Undefined variable '{name}'")]
    UndefinedVariable {
        name: String,
        span: Span,
    },

    /// Invocation of an undefined function or method.
    #[error("Undefined function '{name}'")]
    UndefinedFunction {
        name: String,
        span: Span,
    },

    /// Reference to an undefined type.
    #[error("Undefined type '{name}'")]
    UndefinedType {
        name: String,
        span: Span,
    },

    /// Attempt to reassign an immutable variable.
    #[error("Cannot mutate immutable variable '{name}'")]
    CannotMutateImmutable {
        name: String,
        span: Span,
    },

    /// Attempt to use a value that has already been moved.
    #[error("Use of moved value '{name}'")]
    UseOfMovedValue {
        name: String,
        span: Span,
    },

    /// Non-exhaustive match expression missing required cases.
    #[error("Non-exhaustive match pattern, missing: {missing_cases:?}")]
    NonExhaustiveMatch {
        missing_cases: Vec<String>,
        span: Span,
    },

    /// Invalid operand types for a binary operator.
    #[error("Invalid binary operation '{op}' between '{left}' and '{right}'")]
    InvalidBinaryOp {
        op: BinaryOp,
        left: Type,
        right: Type,
        span: Span,
    },

    /// Invalid operand type for a unary operator.
    #[error("Invalid unary operation '{op}' on type '{target}'")]
    InvalidUnaryOp {
        op: UnaryOp,
        target: Type,
        span: Span,
    },
}

impl TypeError {
    /// Returns the source span where the type error occurred.
    pub fn span(&self) -> Span {
        match self {
            TypeError::TypeMismatch { span, .. }
            | TypeError::UndefinedVariable { span, .. }
            | TypeError::UndefinedFunction { span, .. }
            | TypeError::UndefinedType { span, .. }
            | TypeError::CannotMutateImmutable { span, .. }
            | TypeError::UseOfMovedValue { span, .. }
            | TypeError::NonExhaustiveMatch { span, .. }
            | TypeError::InvalidBinaryOp { span, .. }
            | TypeError::InvalidUnaryOp { span, .. } => *span,
        }
    }
}
