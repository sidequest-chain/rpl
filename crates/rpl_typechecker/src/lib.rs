//! # rpl_typechecker — Semantic Type Checker and Kleene Logic Validator for RPL
//!
//! Enforces strong static typing, Kleene ternary logic algebra, exhaustiveness checking
//! on pattern matching, and ownership move validation across RPL programs.

pub mod checker;
pub mod env;
pub mod error;

pub use checker::TypeChecker;
pub use env::Environment;
pub use error::TypeError;

use rpl_ast::Program;

/// Type checks an RPL AST program, returning all discovered type errors or `Ok(())`.
pub fn check_program(program: &Program) -> Result<(), Vec<TypeError>> {
    let mut checker = TypeChecker::new();
    checker.check_program(program)
}
