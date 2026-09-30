//! Error types for Cranelift JIT compilation and execution.

use thiserror::Error;

/// Error variants encountered during Cranelift JIT compilation or execution.
#[derive(Debug, Error)]
pub enum CodegenCraneliftError {
    /// An error originating from Cranelift module creation or definition.
    #[error("Cranelift module error: {0}")]
    ModuleError(Box<cranelift_module::ModuleError>),

    /// Type mismatch between expected and actual types.
    #[error("Type mismatch: expected {expected}, found {found}")]
    TypeMismatch {
        /// Expected type description.
        expected: String,
        /// Found type description.
        found: String,
    },

    /// An unsupported RPL feature encountered in Cranelift JIT backend.
    #[error("Unsupported feature: {feature}")]
    UnsupportedFeature {
        /// Description of the unsupported feature.
        feature: String,
    },

    /// An undefined variable, function, or symbol name.
    #[error("Undefined symbol: {name}")]
    UndefinedSymbol {
        /// Symbol identifier.
        name: String,
    },

    /// A runtime execution error while invoking JIT-compiled code.
    #[error("JIT execution error: {message}")]
    ExecutionError {
        /// Execution error details.
        message: String,
    },
}

impl From<cranelift_module::ModuleError> for CodegenCraneliftError {
    fn from(err: cranelift_module::ModuleError) -> Self {
        Self::ModuleError(Box::new(err))
    }
}
