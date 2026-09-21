//! AST construction and validation error types.

use crate::span::Span;
use thiserror::Error;

/// Errors that may occur during AST construction or transformation.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum AstError {
    /// An invalid literal value was encountered.
    #[error("Invalid literal at {span}: {message}")]
    InvalidLiteral {
        /// Description of the error.
        message: String,
        /// Source code span where the error occurred.
        span: Span,
    },

    /// AST structural validation error.
    #[error("AST validation error at {span}: {message}")]
    ValidationError {
        /// Description of the validation failure.
        message: String,
        /// Source code span where the error occurred.
        span: Span,
    },
}
