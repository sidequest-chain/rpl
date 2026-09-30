//! C99 transpiler backend for Running Pseudo Language (RPL).

pub mod codegen;
pub mod error;
pub mod runtime;
pub mod types;

use rpl_ast::Program;

pub use codegen::CGenerator;
pub use error::CodegenError;
pub use runtime::RPL_RUNTIME_H;

/// Compiles an RPL AST `Program` into clean, valid C99 source code.
pub fn generate_c(program: &Program) -> Result<String, CodegenError> {
    let mut generator = CGenerator::new();
    generator.generate(program)
}
