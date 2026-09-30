//! In-memory JIT execution engine powered by Cranelift.

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use rpl_ast::Program;

use crate::compiler::Compiler;
use crate::error::CodegenCraneliftError;
use crate::runtime::register_runtime_symbols;

/// In-memory JIT execution manager for RPL programs.
pub struct JITCompiler {
    module: JITModule,
    compiler: Compiler,
}

impl JITCompiler {
    /// Creates a new JIT compiler targeting the native host architecture.
    pub fn new() -> Result<Self, CodegenCraneliftError> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| CodegenCraneliftError::ExecutionError {
                message: format!("Cranelift flag error: {e}"),
            })?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| CodegenCraneliftError::ExecutionError {
                message: format!("Cranelift flag error: {e}"),
            })?;

        let isa_builder =
            cranelift_native::builder().map_err(|msg| CodegenCraneliftError::ExecutionError {
                message: format!("Host ISA builder error: {msg}"),
            })?;

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| CodegenCraneliftError::ExecutionError {
                message: format!("Failed to create host ISA: {e}"),
            })?;

        let ptr_type = isa.pointer_type();
        let mut jit_builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        // Register runtime helper symbols in JIT symbol table
        register_runtime_symbols(&mut jit_builder);

        let module = JITModule::new(jit_builder);
        let compiler = Compiler::new(ptr_type);

        Ok(Self { module, compiler })
    }

    /// Compiles and executes an RPL `Program` in memory, returning the exit code.
    pub fn compile_and_run(&mut self, program: &Program) -> Result<i64, CodegenCraneliftError> {
        let main_func_id = self.compiler.compile_program(&mut self.module, program)?;

        // Finalize all definitions and resolve internal machine code addresses
        self.module.finalize_definitions()?;

        // Retrieve executable function pointer
        let code_ptr = self.module.get_finalized_function(main_func_id);

        let main_fn: extern "C" fn() -> i64 = unsafe { std::mem::transmute(code_ptr) };
        let exit_code = main_fn();

        Ok(exit_code)
    }
}

/// Convenience function to compile and run an RPL AST program via Cranelift JIT.
pub fn run_program(program: &Program) -> Result<i64, CodegenCraneliftError> {
    let mut jit = JITCompiler::new()?;
    jit.compile_and_run(program)
}
