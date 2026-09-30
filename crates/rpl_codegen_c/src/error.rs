//! Error types for RPL C99 code generation.

use thiserror::Error;

/// Errors that may occur during C99 code generation.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CodegenError {
    /// An unexpected AST construct was encountered.
    #[error("Unsupported or unexpected AST construct: {0}")]
    UnsupportedConstruct(String),

    /// A type error or unresolved type occurred during code generation.
    #[error("Type resolution failed for identifier '{0}': {1}")]
    TypeResolutionError(String, String),

    /// General code generation failure.
    #[error("Code generation error: {0}")]
    Generic(String),
}
