//! Cranelift JIT code generator for RPL.

pub mod error;
pub mod runtime;
pub mod types;
pub mod compiler;
pub mod jit;

pub use error::CodegenCraneliftError;
pub use jit::run_program;
