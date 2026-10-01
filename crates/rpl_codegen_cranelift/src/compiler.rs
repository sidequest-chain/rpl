//! Core AST lowering and Cranelift IR code generation.

use std::collections::HashMap;

use cranelift::prelude::*;
use cranelift_frontend::{FunctionBuilder, FunctionBuilderContext, Variable};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};

use rpl_ast::{
    BinaryOp, Block, Expr, InterpolationFragment, Literal, Pattern, Program, Stmt,
    TritValue, Type, UnaryOp,
};

use crate::error::CodegenCraneliftError;
use crate::types::{compute_struct_layout, rpl_to_cl_type, StructLayout};

/// Metadata for a declared function.
#[derive(Debug, Clone)]
pub struct FnMeta {
    /// Cranelift Function ID.
    pub func_id: FuncId,
    /// Parameter names and RPL types.
    pub params: Vec<(String, Type)>,
    /// Return RPL type.
    pub return_type: Option<Type>,
}

/// Compiler state managing Cranelift module generation.
pub struct Compiler {
    /// Composite struct layouts.
    pub structs: HashMap<String, StructLayout>,
    /// Declared functions metadata.
    pub functions: HashMap<String, FnMeta>,
    /// Pointer type for the target ISA (typically I64 on x86_64/AArch64).
    pub ptr_type: types::Type,
    /// String literals mapped to Cranelift data definitions.
    pub string_literals: HashMap<String, cranelift_module::DataId>,
    /// Variable counter for generating unique Variable indices.
    pub var_counter: u32,
    /// Map of variable names to their Cranelift Variable handle and RPL type in current function.
    pub local_vars: HashMap<String, (Variable, Type)>,
}

/// Checks whether the currently active block has already been terminated by a branch or return.
fn is_block_filled(builder: &FunctionBuilder) -> bool {
    if let Some(block) = builder.current_block() {
        if let Some(inst) = builder.func.layout.last_inst(block) {
            return builder.func.dfg.insts[inst].opcode().is_terminator();
        }
    }
    false
}

impl Compiler {
    /// Creates a new compiler instance.
    pub fn new(ptr_type: types::Type) -> Self {
        Self {
            structs: HashMap::new(),
            functions: HashMap::new(),
            ptr_type,
            string_literals: HashMap::new(),
            var_counter: 0,
            local_vars: HashMap::new(),
        }
    }

    /// Allocates a new unique Variable in Cranelift function builder.
    pub fn next_var(&mut self) -> Variable {
        let v = Variable::new(self.var_counter as usize);
        self.var_counter += 1;
        v
    }

    /// Compiles an entire RPL Program into the target module and returns the entrypoint `main` FuncId.
    pub fn compile_program<M: Module>(
        &mut self,
        module: &mut M,
        program: &Program,
    ) -> Result<FuncId, CodegenCraneliftError> {
        // Pass 1: Collect struct type declarations and compute memory layouts
        for stmt in &program.statements {
            if let Stmt::TypeDecl { name, fields, .. } = stmt {
                let layout = compute_struct_layout(name, fields, self.ptr_type, &self.structs);
                self.structs.insert(name.clone(), layout);
            }
        }

        // Pass 2: Declare all user functions and runtime functions in the module
        self.declare_runtime_signatures(module)?;

        for stmt in &program.statements {
            if let Stmt::FnDecl {
                name,
                params,
                return_type,
                ..
            } = stmt
            {
                let mut sig = module.make_signature();
                for p in params {
                    let cl_ty = rpl_to_cl_type(&p.param_type, self.ptr_type);
                    sig.params.push(AbiParam::new(cl_ty));
                }
                if let Some(ret) = return_type {
                    let cl_ret = rpl_to_cl_type(ret, self.ptr_type);
                    sig.returns.push(AbiParam::new(cl_ret));
                }

                let fn_name = if name == "main" { "rpl_user_main" } else { name.as_str() };
                let func_id = module.declare_function(fn_name, Linkage::Export, &sig)?;
                let param_metas = params
                    .iter()
                    .map(|p| (p.name.clone(), p.param_type.clone()))
                    .collect();
                self.functions.insert(
                    name.clone(),
                    FnMeta {
                        func_id,
                        params: param_metas,
                        return_type: return_type.clone(),
                    },
                );
            }
        }

        // Declare top-level main entrypoint function: fn() -> i64
        let mut main_sig = module.make_signature();
        main_sig.returns.push(AbiParam::new(types::I64));
        let main_func_id = module.declare_function("main", Linkage::Export, &main_sig)?;

        // Pass 3: Compile user functions
        for stmt in &program.statements {
            if let Stmt::FnDecl {
                name,
                params,
                return_type,
                body,
                ..
            } = stmt
            {
                self.compile_user_fn(module, name, params, return_type.as_ref(), body)?;
            }
        }

        // Pass 4: Compile top-level statements into main entrypoint function
        self.compile_main_fn(module, main_func_id, program)?;

        Ok(main_func_id)
    }

    /// Declares all external runtime functions in the Cranelift module.
    fn declare_runtime_signatures<M: Module>(
        &mut self,
        module: &mut M,
    ) -> Result<(), CodegenCraneliftError> {
        let ptr_ty = self.ptr_type;

        // print_int: (i64) -> void
        let mut sig_int = module.make_signature();
        sig_int.params.push(AbiParam::new(types::I64));
        module.declare_function("rpl_jit_print_int", Linkage::Import, &sig_int)?;

        // print_float: (f64) -> void
        let mut sig_float = module.make_signature();
        sig_float.params.push(AbiParam::new(types::F64));
        module.declare_function("rpl_jit_print_float", Linkage::Import, &sig_float)?;

        // print_bool: (i8) -> void
        let mut sig_bool = module.make_signature();
        sig_bool.params.push(AbiParam::new(types::I8));
        module.declare_function("rpl_jit_print_bool", Linkage::Import, &sig_bool)?;

        // print_trit: (i8) -> void
        let mut sig_trit = module.make_signature();
        sig_trit.params.push(AbiParam::new(types::I8));
        module.declare_function("rpl_jit_print_trit", Linkage::Import, &sig_trit)?;

        // print_str: (ptr, i64) -> void
        let mut sig_str = module.make_signature();
        sig_str.params.push(AbiParam::new(ptr_ty));
        sig_str.params.push(AbiParam::new(types::I64));
        module.declare_function("rpl_jit_print_str", Linkage::Import, &sig_str)?;

        // str_concat: (ptr, i64, ptr, i64) -> ptr
        let mut sig_concat = module.make_signature();
        sig_concat.params.push(AbiParam::new(ptr_ty));
        sig_concat.params.push(AbiParam::new(types::I64));
        sig_concat.params.push(AbiParam::new(ptr_ty));
        sig_concat.params.push(AbiParam::new(types::I64));
        sig_concat.returns.push(AbiParam::new(ptr_ty));
        module.declare_function("rpl_jit_str_concat", Linkage::Import, &sig_concat)?;

        // int_to_str: (i64) -> ptr
        let mut sig_i2s = module.make_signature();
        sig_i2s.params.push(AbiParam::new(types::I64));
        sig_i2s.returns.push(AbiParam::new(ptr_ty));
        module.declare_function("rpl_jit_int_to_str", Linkage::Import, &sig_i2s)?;

        // float_to_str: (f64) -> ptr
        let mut sig_f2s = module.make_signature();
        sig_f2s.params.push(AbiParam::new(types::F64));
        sig_f2s.returns.push(AbiParam::new(ptr_ty));
        module.declare_function("rpl_jit_float_to_str", Linkage::Import, &sig_f2s)?;

        // bool_to_str: (i8) -> ptr
        let mut sig_b2s = module.make_signature();
        sig_b2s.params.push(AbiParam::new(types::I8));
        sig_b2s.returns.push(AbiParam::new(ptr_ty));
        module.declare_function("rpl_jit_bool_to_str", Linkage::Import, &sig_b2s)?;

        // trit_to_str: (i8) -> ptr
        let mut sig_t2s = module.make_signature();
        sig_t2s.params.push(AbiParam::new(types::I8));
        sig_t2s.returns.push(AbiParam::new(ptr_ty));
        module.declare_function("rpl_jit_trit_to_str", Linkage::Import, &sig_t2s)?;

        // str_len: (ptr) -> i64
        let mut sig_slen = module.make_signature();
        sig_slen.params.push(AbiParam::new(ptr_ty));
        sig_slen.returns.push(AbiParam::new(types::I64));
        module.declare_function("rpl_jit_str_len", Linkage::Import, &sig_slen)?;

        // input: () -> ptr
        let mut sig_input = module.make_signature();
        sig_input.returns.push(AbiParam::new(ptr_ty));
        module.declare_function("rpl_jit_input", Linkage::Import, &sig_input)?;

        // read_file: (ptr) -> ptr
        let mut sig_read_file = module.make_signature();
        sig_read_file.params.push(AbiParam::new(ptr_ty));
        sig_read_file.returns.push(AbiParam::new(ptr_ty));
        module.declare_function("rpl_jit_read_file", Linkage::Import, &sig_read_file)?;

        // write_file: (ptr, ptr) -> i8
        let mut sig_write_file = module.make_signature();
        sig_write_file.params.push(AbiParam::new(ptr_ty));
        sig_write_file.params.push(AbiParam::new(ptr_ty));
        sig_write_file.returns.push(AbiParam::new(types::I8));
        module.declare_function("rpl_jit_write_file", Linkage::Import, &sig_write_file)?;

        // append_file: (ptr, ptr) -> i8
        let mut sig_append_file = module.make_signature();
        sig_append_file.params.push(AbiParam::new(ptr_ty));
        sig_append_file.params.push(AbiParam::new(ptr_ty));
        sig_append_file.returns.push(AbiParam::new(types::I8));
        module.declare_function("rpl_jit_append_file", Linkage::Import, &sig_append_file)?;

        // open_file: (ptr, ptr) -> ptr
        let mut sig_open_file = module.make_signature();
        sig_open_file.params.push(AbiParam::new(ptr_ty));
        sig_open_file.params.push(AbiParam::new(ptr_ty));
        sig_open_file.returns.push(AbiParam::new(ptr_ty));
        module.declare_function("rpl_jit_open_file", Linkage::Import, &sig_open_file)?;

        // read_line: (ptr) -> ptr
        let mut sig_read_line = module.make_signature();
        sig_read_line.params.push(AbiParam::new(ptr_ty));
        sig_read_line.returns.push(AbiParam::new(ptr_ty));
        module.declare_function("rpl_jit_read_line", Linkage::Import, &sig_read_line)?;

        // write_line: (ptr, ptr) -> i8
        let mut sig_write_line = module.make_signature();
        sig_write_line.params.push(AbiParam::new(ptr_ty));
        sig_write_line.params.push(AbiParam::new(ptr_ty));
        sig_write_line.returns.push(AbiParam::new(types::I8));
        module.declare_function("rpl_jit_write_line", Linkage::Import, &sig_write_line)?;

        // close_file: (ptr) -> void
        let mut sig_close_file = module.make_signature();
        sig_close_file.params.push(AbiParam::new(ptr_ty));
        module.declare_function("rpl_jit_close_file", Linkage::Import, &sig_close_file)?;

        Ok(())
    }

    /// Compiles a user-defined function.
    fn compile_user_fn<M: Module>(
        &mut self,
        module: &mut M,
        name: &str,
        params: &[rpl_ast::Param],
        return_type: Option<&Type>,
        body: &Block,
    ) -> Result<(), CodegenCraneliftError> {
        let meta = self
            .functions
            .get(name)
            .cloned()
            .ok_or_else(|| CodegenCraneliftError::UndefinedSymbol {
                name: name.to_string(),
            })?;

        let mut cl_ctx = module.make_context();
        let mut sig = module.make_signature();
        for p in params {
            let cl_ty = rpl_to_cl_type(&p.param_type, self.ptr_type);
            sig.params.push(AbiParam::new(cl_ty));
        }
        if let Some(ret) = return_type {
            let cl_ret = rpl_to_cl_type(ret, self.ptr_type);
            sig.returns.push(AbiParam::new(cl_ret));
        }
        cl_ctx.func.signature = sig;

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut cl_ctx.func, &mut fn_builder_ctx);
        let entry_block = builder.create_block();
        builder.append_block_params_for_function_params(entry_block);
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        self.local_vars.clear();
        self.var_counter = 0;

        // Bind incoming arguments to local variables
        for (i, p) in params.iter().enumerate() {
            let val = builder.block_params(entry_block)[i];
            let var = self.next_var();
            let cl_ty = rpl_to_cl_type(&p.param_type, self.ptr_type);
            builder.declare_var(var, cl_ty);
            builder.def_var(var, val);
            self.local_vars.insert(p.name.clone(), (var, p.param_type.clone()));
        }

        // Lower body statements
        for stmt in &body.stmts {
            self.compile_stmt(module, &mut builder, stmt)?;
        }

        // Implicit return if block is not yet filled with a terminator
        if !is_block_filled(&builder) {
            if let Some(ret_ty) = &meta.return_type {
                let default_ret = match ret_ty {
                    Type::Float | Type::Float32 => builder.ins().f64const(0.0),
                    Type::Bool | Type::Trit => builder.ins().iconst(types::I8, 0),
                    _ => builder.ins().iconst(self.ptr_type, 0),
                };
                builder.ins().return_(&[default_ret]);
            } else {
                builder.ins().return_(&[]);
            }
        }

        builder.finalize();
        module.define_function(meta.func_id, &mut cl_ctx)?;
        Ok(())
    }

    /// Compiles top-level statements into the main entrypoint function.
    fn compile_main_fn<M: Module>(
        &mut self,
        module: &mut M,
        main_func_id: FuncId,
        program: &Program,
    ) -> Result<(), CodegenCraneliftError> {
        let mut cl_ctx = module.make_context();
        let mut sig = module.make_signature();
        sig.returns.push(AbiParam::new(types::I64));
        cl_ctx.func.signature = sig;

        let mut fn_builder_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut cl_ctx.func, &mut fn_builder_ctx);
        let entry_block = builder.create_block();
        builder.switch_to_block(entry_block);
        builder.seal_block(entry_block);

        self.local_vars.clear();
        self.var_counter = 0;

        let mut last_val = None;

        // Lower top-level non-declaration statements
        for stmt in &program.statements {
            match stmt {
                Stmt::TypeDecl { .. } | Stmt::FnDecl { .. } => {
                    // Handled in pre-passes
                }
                Stmt::Expr(expr) => {
                    let v = self.compile_expr(module, &mut builder, expr, None)?;
                    last_val = Some(v);
                }
                _ => {
                    self.compile_stmt(module, &mut builder, stmt)?;
                }
            }
        }

        // If user defined a parameterless `main()`, invoke it
        if let Some(meta) = self.functions.get("main") {
            let local_fn = module.declare_func_in_func(meta.func_id, builder.func);
            let call_inst = builder.ins().call(local_fn, &[]);
            let results = builder.inst_results(call_inst);
            if let Some(r) = results.first() {
                last_val = Some(*r);
            }
        }

        if !is_block_filled(&builder) {
            let ret_val = match last_val {
                Some(v) => {
                    let ty = builder.func.dfg.value_type(v);
                    if ty == types::I64 {
                        v
                    } else if ty.is_int() {
                        builder.ins().sextend(types::I64, v)
                    } else {
                        builder.ins().iconst(types::I64, 0)
                    }
                }
                None => builder.ins().iconst(types::I64, 0),
            };
            builder.ins().return_(&[ret_val]);
        }

        builder.finalize();
        module.define_function(main_func_id, &mut cl_ctx)?;
        Ok(())
    }

    /// Compiles a single statement.
    pub fn compile_stmt<M: Module>(
        &mut self,
        module: &mut M,
        builder: &mut FunctionBuilder,
        stmt: &Stmt,
    ) -> Result<(), CodegenCraneliftError> {
        match stmt {
            Stmt::Let {
                name,
                type_annot,
                value,
                ..
            }
            | Stmt::MutLet {
                name,
                type_annot,
                value,
                ..
            } => {
                let inferred_ty = type_annot
                    .clone()
                    .unwrap_or_else(|| self.infer_expr_type(value));
                let val = self.compile_expr(module, builder, value, Some(&inferred_ty))?;
                let cl_ty = builder.func.dfg.value_type(val);

                let var = self.next_var();
                builder.declare_var(var, cl_ty);
                builder.def_var(var, val);

                self.local_vars.insert(name.clone(), (var, inferred_ty));
                Ok(())
            }

            Stmt::Assign { target, value, .. } => {
                match target {
                    Expr::Identifier(name, _) => {
                        let (var, var_ty) = self
                            .local_vars
                            .get(name)
                            .cloned()
                            .ok_or_else(|| CodegenCraneliftError::UndefinedSymbol {
                                name: name.clone(),
                            })?;
                        let val = self.compile_expr(module, builder, value, Some(&var_ty))?;
                        builder.def_var(var, val);
                        Ok(())
                    }
                    Expr::MemberAccess { target, field, .. } => {
                        // Store to struct field: *(target_ptr + offset) = val
                        let target_ty = self.infer_expr_type(target);
                        let target_ptr = self.compile_expr(module, builder, target, None)?;

                        if let Type::Named(s_name) = &target_ty {
                            if let Some(layout) = self.structs.get(s_name).cloned() {
                                if let Some(flayout) = layout.get_field(field) {
                                    let val = self.compile_expr(
                                        module,
                                        builder,
                                        value,
                                        Some(&flayout.rpl_type),
                                    )?;
                                    builder.ins().store(
                                        MemFlags::trusted(),
                                        val,
                                        target_ptr,
                                        flayout.offset as i32,
                                    );
                                    return Ok(());
                                }
                            }
                        }

                        Err(CodegenCraneliftError::UnsupportedFeature {
                            feature: format!("Assignment to member '{field}' on non-struct"),
                        })
                    }
                    _ => Err(CodegenCraneliftError::UnsupportedFeature {
                        feature: "Complex assignment target".to_string(),
                    }),
                }
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_val = self.compile_expr(module, builder, condition, None)?;
                let cond_ty = builder.func.dfg.value_type(cond_val);

                // Convert condition to boolean flag (1 = true)
                let is_true = if cond_ty == types::I8 {
                    let one = builder.ins().iconst(types::I8, 1);
                    builder.ins().icmp(IntCC::Equal, cond_val, one)
                } else if cond_ty == types::I64 {
                    let zero = builder.ins().iconst(types::I64, 0);
                    builder.ins().icmp(IntCC::NotEqual, cond_val, zero)
                } else {
                    cond_val
                };

                let then_block = builder.create_block();
                let else_block = builder.create_block();
                let merge_block = builder.create_block();

                builder
                    .ins()
                    .brif(is_true, then_block, &[], else_block, &[]);

                // Then branch
                builder.switch_to_block(then_block);
                builder.seal_block(then_block);
                for s in &then_branch.stmts {
                    self.compile_stmt(module, builder, s)?;
                }
                if !is_block_filled(builder) {
                    builder.ins().jump(merge_block, &[]);
                }

                // Else branch
                builder.switch_to_block(else_block);
                builder.seal_block(else_block);
                if let Some(else_b) = else_branch {
                    for s in &else_b.stmts {
                        self.compile_stmt(module, builder, s)?;
                    }
                }
                if !is_block_filled(builder) {
                    builder.ins().jump(merge_block, &[]);
                }

                // Merge block
                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);
                Ok(())
            }

            Stmt::Match { subject, cases, .. } => {
                let subject_val = self.compile_expr(module, builder, subject, None)?;
                let subject_cl_ty = builder.func.dfg.value_type(subject_val);

                let merge_block = builder.create_block();

                // Chain through cases sequentially
                let mut current_test_block = builder.current_block().unwrap();

                for (idx, case) in cases.iter().enumerate() {
                    let case_body_block = builder.create_block();
                    let next_test_block = if idx + 1 < cases.len() {
                        builder.create_block()
                    } else {
                        merge_block
                    };

                    if idx > 0 {
                        builder.switch_to_block(current_test_block);
                        builder.seal_block(current_test_block);
                    }

                    // Test pattern match condition
                    match &case.pattern {
                        Pattern::Literal(lit, _) => {
                            let expected_v = match lit {
                                Literal::Trit(t) => match t {
                                    TritValue::True => 1i64,
                                    TritValue::False => -1i64,
                                    TritValue::Unknown => 0i64,
                                },
                                Literal::Bool(b) => {
                                    if subject_cl_ty == types::I8 {
                                        if *b { 1i64 } else { -1i64 }
                                    } else {
                                        if *b { 1i64 } else { 0i64 }
                                    }
                                }
                                Literal::Int(v) => *v,
                                _ => 0i64,
                            };
                            let expected_val = builder.ins().iconst(subject_cl_ty, expected_v);
                            let matched =
                                builder.ins().icmp(IntCC::Equal, subject_val, expected_val);
                            builder
                                .ins()
                                .brif(matched, case_body_block, &[], next_test_block, &[]);
                        }
                        Pattern::Wildcard(_) | Pattern::Identifier(_, _) => {
                            // Default wildcard always matches
                            builder.ins().jump(case_body_block, &[]);
                        }
                        _ => {
                            builder.ins().jump(next_test_block, &[]);
                        }
                    }

                    // Compile case body
                    builder.switch_to_block(case_body_block);
                    builder.seal_block(case_body_block);

                    for s in &case.body.stmts {
                        self.compile_stmt(module, builder, s)?;
                    }

                    if !is_block_filled(builder) {
                        builder.ins().jump(merge_block, &[]);
                    }

                    current_test_block = next_test_block;
                }

                builder.switch_to_block(merge_block);
                builder.seal_block(merge_block);
                Ok(())
            }

            Stmt::For {
                item_name,
                iterator,
                body,
                ..
            } => match iterator {
                Expr::Range { start, end, .. } => {
                    let start_val = self.compile_expr(module, builder, start, Some(&Type::Int))?;
                    let end_val = self.compile_expr(module, builder, end, Some(&Type::Int))?;

                    let loop_var = self.next_var();
                    builder.declare_var(loop_var, types::I64);
                    builder.def_var(loop_var, start_val);
                    self.local_vars
                        .insert(item_name.clone(), (loop_var, Type::Int));

                    let header_block = builder.create_block();
                    let body_block = builder.create_block();
                    let exit_block = builder.create_block();

                    builder.ins().jump(header_block, &[]);

                    // Header block: condition test (current <= end)
                    builder.switch_to_block(header_block);
                    let current = builder.use_var(loop_var);
                    let cond =
                        builder
                            .ins()
                            .icmp(IntCC::SignedLessThanOrEqual, current, end_val);
                    builder
                        .ins()
                        .brif(cond, body_block, &[], exit_block, &[]);

                    // Body block: execute body statements and increment
                    builder.switch_to_block(body_block);
                    builder.seal_block(body_block);

                    for s in &body.stmts {
                        self.compile_stmt(module, builder, s)?;
                    }

                    if !is_block_filled(builder) {
                        let cur_val = builder.use_var(loop_var);
                        let next_val = builder.ins().iadd_imm(cur_val, 1);
                        builder.def_var(loop_var, next_val);
                        builder.ins().jump(header_block, &[]);
                    }

                    builder.seal_block(header_block);

                    // Exit block
                    builder.switch_to_block(exit_block);
                    builder.seal_block(exit_block);
                    Ok(())
                }
                _ => Err(CodegenCraneliftError::UnsupportedFeature {
                    feature: "For loops over non-range iterators".to_string(),
                }),
            },

            Stmt::Return(opt_expr, _) => {
                match opt_expr {
                    Some(expr) => {
                        let val = self.compile_expr(module, builder, expr, None)?;
                        builder.ins().return_(&[val]);
                    }
                    None => {
                        builder.ins().return_(&[]);
                    }
                }
                Ok(())
            }

            Stmt::Expr(expr) => {
                self.compile_expr(module, builder, expr, None)?;
                Ok(())
            }

            Stmt::ParallelFor { body, .. } | Stmt::Spawn { body, .. } => {
                // Sequential fallback for structured concurrency blocks
                for s in &body.stmts {
                    self.compile_stmt(module, builder, s)?;
                }
                Ok(())
            }

            Stmt::TypeDecl { .. } | Stmt::FnDecl { .. } => Ok(()),
        }
    }

    /// Compiles an expression into a Cranelift Value.
    pub fn compile_expr<M: Module>(
        &mut self,
        module: &mut M,
        builder: &mut FunctionBuilder,
        expr: &Expr,
        expected_type: Option<&Type>,
    ) -> Result<Value, CodegenCraneliftError> {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Int(v) => Ok(builder.ins().iconst(types::I64, *v)),
                Literal::Float(v) => Ok(builder.ins().f64const(*v)),
                Literal::Bool(b) => {
                    if expected_type == Some(&Type::Trit) {
                        // Trit representation: true = 1, false = -1
                        let val = if *b { 1i64 } else { -1i64 };
                        Ok(builder.ins().iconst(types::I8, val))
                    } else {
                        let val = if *b { 1i64 } else { 0i64 };
                        Ok(builder.ins().iconst(types::I8, val))
                    }
                }
                Literal::Trit(t) => {
                    let val = match t {
                        TritValue::True => 1i64,
                        TritValue::False => -1i64,
                        TritValue::Unknown => 0i64,
                    };
                    Ok(builder.ins().iconst(types::I8, val))
                }
                Literal::String(s) => self.get_string_constant_ptr(module, builder, s),
            },

            Expr::Identifier(name, _) => {
                if let Some((var, _)) = self.local_vars.get(name) {
                    Ok(builder.use_var(*var))
                } else {
                    Err(CodegenCraneliftError::UndefinedSymbol { name: name.clone() })
                }
            }

            Expr::Binary {
                left, op, right, ..
            } => {
                let left_ty = self.infer_expr_type(left);
                let right_ty = self.infer_expr_type(right);
                let is_trit = expected_type == Some(&Type::Trit)
                    || left_ty == Type::Trit
                    || right_ty == Type::Trit;

                let op_exp = if is_trit { Some(&Type::Trit) } else { None };
                let l_val = self.compile_expr(module, builder, left, op_exp)?;
                let r_val = self.compile_expr(module, builder, right, op_exp)?;

                let l_cl_ty = builder.func.dfg.value_type(l_val);
                let is_float = l_cl_ty.is_float();

                match op {
                    // Arithmetic
                    BinaryOp::Add => {
                        if is_float {
                            Ok(builder.ins().fadd(l_val, r_val))
                        } else {
                            Ok(builder.ins().iadd(l_val, r_val))
                        }
                    }
                    BinaryOp::Sub => {
                        if is_float {
                            Ok(builder.ins().fsub(l_val, r_val))
                        } else {
                            Ok(builder.ins().isub(l_val, r_val))
                        }
                    }
                    BinaryOp::Mul => {
                        if is_float {
                            Ok(builder.ins().fmul(l_val, r_val))
                        } else {
                            Ok(builder.ins().imul(l_val, r_val))
                        }
                    }
                    BinaryOp::Div => {
                        if is_float {
                            Ok(builder.ins().fdiv(l_val, r_val))
                        } else {
                            Ok(builder.ins().sdiv(l_val, r_val))
                        }
                    }
                    BinaryOp::Mod => Ok(builder.ins().srem(l_val, r_val)),

                    // Comparisons (return I8 in Cranelift)
                    BinaryOp::Eq => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::Equal, l_val, r_val))
                        } else {
                            Ok(builder.ins().icmp(IntCC::Equal, l_val, r_val))
                        }
                    }
                    BinaryOp::NotEq => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::NotEqual, l_val, r_val))
                        } else {
                            Ok(builder.ins().icmp(IntCC::NotEqual, l_val, r_val))
                        }
                    }
                    BinaryOp::Lt => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::LessThan, l_val, r_val))
                        } else {
                            Ok(builder.ins().icmp(IntCC::SignedLessThan, l_val, r_val))
                        }
                    }
                    BinaryOp::LtEq => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::LessThanOrEqual, l_val, r_val))
                        } else {
                            Ok(builder.ins().icmp(IntCC::SignedLessThanOrEqual, l_val, r_val))
                        }
                    }
                    BinaryOp::Gt => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::GreaterThan, l_val, r_val))
                        } else {
                            Ok(builder.ins().icmp(IntCC::SignedGreaterThan, l_val, r_val))
                        }
                    }
                    BinaryOp::GtEq => {
                        if is_float {
                            Ok(builder.ins().fcmp(FloatCC::GreaterThanOrEqual, l_val, r_val))
                        } else {
                            Ok(builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, l_val, r_val))
                        }
                    }

                    // Logical / Kleene 3-state logic
                    BinaryOp::And => {
                        if is_trit {
                            // Kleene AND = min(l, r) via SignedLessThan + select
                            let is_slt =
                                builder.ins().icmp(IntCC::SignedLessThan, l_val, r_val);
                            Ok(builder.ins().select(is_slt, l_val, r_val))
                        } else {
                            Ok(builder.ins().band(l_val, r_val))
                        }
                    }
                    BinaryOp::Or => {
                        if is_trit {
                            // Kleene OR = max(l, r) via SignedGreaterThan + select
                            let is_sgt =
                                builder.ins().icmp(IntCC::SignedGreaterThan, l_val, r_val);
                            Ok(builder.ins().select(is_sgt, l_val, r_val))
                        } else {
                            Ok(builder.ins().bor(l_val, r_val))
                        }
                    }

                    // Bitwise
                    BinaryOp::BitAnd => Ok(builder.ins().band(l_val, r_val)),
                    BinaryOp::BitOr => Ok(builder.ins().bor(l_val, r_val)),
                    BinaryOp::BitXor => Ok(builder.ins().bxor(l_val, r_val)),
                    BinaryOp::Shl => Ok(builder.ins().ishl(l_val, r_val)),
                    BinaryOp::Shr => Ok(builder.ins().sshr(l_val, r_val)),
                }
            }

            Expr::Unary { op, expr, .. } => {
                let expr_ty = self.infer_expr_type(expr);
                let is_trit = expected_type == Some(&Type::Trit) || expr_ty == Type::Trit;

                let val = self.compile_expr(
                    module,
                    builder,
                    expr,
                    if is_trit { Some(&Type::Trit) } else { None },
                )?;
                let cl_ty = builder.func.dfg.value_type(val);

                match op {
                    UnaryOp::Not => {
                        if is_trit {
                            // Kleene NOT = -val (-1 -> 1, 0 -> 0, 1 -> -1)
                            Ok(builder.ins().ineg(val))
                        } else if self.is_integer_type(&expr_ty) {
                            Ok(builder.ins().bnot(val))
                        } else {
                            let zero = builder.ins().iconst(cl_ty, 0);
                            let cmp = builder.ins().icmp(IntCC::Equal, val, zero);
                            if cl_ty == types::I8 {
                                Ok(cmp)
                            } else {
                                Ok(builder.ins().uextend(cl_ty, cmp))
                            }
                        }
                    }
                    UnaryOp::Neg => {
                        if cl_ty.is_float() {
                            Ok(builder.ins().fneg(val))
                        } else {
                            Ok(builder.ins().ineg(val))
                        }
                    }
                    UnaryOp::BitNot => Ok(builder.ins().bnot(val)),
                }
            }

            Expr::Pipe { left, right, .. } => {
                let right_fn_name = match right.as_ref() {
                    Expr::Identifier(name, _) => Some(name.as_str()),
                    Expr::Call { callee, .. } => {
                        if let Expr::Identifier(name, _) = callee.as_ref() {
                            Some(name.as_str())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if right_fn_name == Some("print") || right_fn_name == Some("println") {
                    return self.compile_print_call(module, builder, &[left.as_ref().clone()]);
                }

                // Lower pipe `left |> right`: passes left as first parameter to right call
                let left_val = self.compile_expr(module, builder, left, None)?;
                match right.as_ref() {
                    Expr::Call { callee, args, .. } => {
                        let mut all_args = vec![left_val];
                        for a in args {
                            all_args.push(self.compile_expr(module, builder, a, None)?);
                        }
                        self.compile_direct_call(module, builder, callee, all_args)
                    }
                    _ => self.compile_direct_call(module, builder, right, vec![left_val]),
                }
            }

            Expr::Call { callee, args, .. } => {
                if let Expr::Identifier(fn_name, _) = callee.as_ref() {
                    // Check built-in print / println
                    if fn_name == "print" || fn_name == "println" {
                        return self.compile_print_call(module, builder, args);
                    }

                    // Check struct constructor `TypeName(arg1, arg2...)`
                    if self.structs.contains_key(fn_name) {
                        return self.compile_struct_constructor(module, builder, fn_name, args);
                    }
                }

                let mut arg_vals = Vec::new();
                for a in args {
                    arg_vals.push(self.compile_expr(module, builder, a, None)?);
                }
                self.compile_direct_call(module, builder, callee, arg_vals)
            }

            Expr::MemberAccess { target, field, .. } => {
                let target_ty = self.infer_expr_type(target);
                let target_ptr = self.compile_expr(module, builder, target, None)?;

                if let Type::Named(s_name) = &target_ty {
                    if let Some(layout) = self.structs.get(s_name).cloned() {
                        if let Some(flayout) = layout.get_field(field) {
                            let loaded = builder.ins().load(
                                flayout.cl_type,
                                MemFlags::trusted(),
                                target_ptr,
                                flayout.offset as i32,
                            );
                            return Ok(loaded);
                        }
                    }
                }

                Err(CodegenCraneliftError::UnsupportedFeature {
                    feature: format!("Field access '{field}' on non-struct type"),
                })
            }

            Expr::StructBlockInit { name, fields, .. } => {
                let layout = self
                    .structs
                    .get(name)
                    .cloned()
                    .ok_or_else(|| CodegenCraneliftError::UndefinedSymbol { name: name.clone() })?;

                // Allocate struct on stack slot
                let align_shift = layout.alignment.max(1).trailing_zeros() as u8;
                let slot_data =
                    StackSlotData::new(StackSlotKind::ExplicitSlot, layout.total_size, align_shift);
                let slot = builder.create_sized_stack_slot(slot_data);
                let slot_addr = builder.ins().stack_addr(self.ptr_type, slot, 0);

                // Store field values
                for (fname, fexpr) in fields {
                    if let Some(flayout) = layout.get_field(fname) {
                        let fval = self.compile_expr(
                            module,
                            builder,
                            fexpr,
                            Some(&flayout.rpl_type),
                        )?;
                        builder.ins().store(
                            MemFlags::trusted(),
                            fval,
                            slot_addr,
                            flayout.offset as i32,
                        );
                    }
                }

                Ok(slot_addr)
            }

            Expr::StringInterpolation { fragments, .. } => {
                self.compile_string_interpolation(module, builder, fragments)
            }

            _ => Err(CodegenCraneliftError::UnsupportedFeature {
                feature: format!("Expression {expr:?}"),
            }),
        }
    }

    /// Compiles a direct call to a function or runtime helper.
    fn compile_direct_call<M: Module>(
        &mut self,
        module: &mut M,
        builder: &mut FunctionBuilder,
        callee: &Expr,
        args: Vec<Value>,
    ) -> Result<Value, CodegenCraneliftError> {
        if let Expr::Identifier(fn_name, _) = callee {
            if let Some(meta) = self.functions.get(fn_name).cloned() {
                let local_fn = module.declare_func_in_func(meta.func_id, builder.func);
                let call_inst = builder.ins().call(local_fn, &args);
                let results = builder.inst_results(call_inst);
                if let Some(res) = results.first() {
                    return Ok(*res);
                } else {
                    return Ok(builder.ins().iconst(types::I64, 0));
                }
            }

            let runtime_fn_name = match fn_name.as_str() {
                "input" => Some("rpl_jit_input"),
                "read_file" => Some("rpl_jit_read_file"),
                "write_file" => Some("rpl_jit_write_file"),
                "append_file" => Some("rpl_jit_append_file"),
                "open_file" => Some("rpl_jit_open_file"),
                "read_line" => Some("rpl_jit_read_line"),
                "write_line" => Some("rpl_jit_write_line"),
                "close_file" => Some("rpl_jit_close_file"),
                _ => None,
            };

            if let Some(rt_name) = runtime_fn_name {
                let fn_id = module
                    .get_name(rt_name)
                    .and_then(|f| match f {
                        cranelift_module::FuncOrDataId::Func(id) => Some(id),
                        _ => None,
                    })
                    .ok_or_else(|| CodegenCraneliftError::UndefinedSymbol {
                        name: rt_name.to_string(),
                    })?;
                let local_fn = module.declare_func_in_func(fn_id, builder.func);
                let call_inst = builder.ins().call(local_fn, &args);
                let results = builder.inst_results(call_inst);
                if let Some(res) = results.first() {
                    return Ok(*res);
                } else {
                    return Ok(builder.ins().iconst(types::I64, 0));
                }
            }
        }

        Err(CodegenCraneliftError::UndefinedSymbol {
            name: format!("{callee:?}"),
        })
    }

    /// Compiles built-in print / println invocations.
    fn compile_print_call<M: Module>(
        &mut self,
        module: &mut M,
        builder: &mut FunctionBuilder,
        args: &[Expr],
    ) -> Result<Value, CodegenCraneliftError> {
        if args.is_empty() {
            // Print empty newline
            let empty_ptr = self.get_string_constant_ptr(module, builder, "")?;
            let len = builder.ins().iconst(types::I64, 0);
            let fn_id = module
                .get_name("rpl_jit_print_str")
                .and_then(|f| match f {
                    cranelift_module::FuncOrDataId::Func(id) => Some(id),
                    _ => None,
                })
                .ok_or_else(|| CodegenCraneliftError::UndefinedSymbol {
                    name: "rpl_jit_print_str".into(),
                })?;
            let local_fn = module.declare_func_in_func(fn_id, builder.func);
            builder.ins().call(local_fn, &[empty_ptr, len]);
            return Ok(builder.ins().iconst(types::I64, 0));
        }

        let arg = &args[0];
        let arg_ty = self.infer_expr_type(arg);

        match arg_ty {
            Type::Int => {
                let v = self.compile_expr(module, builder, arg, Some(&Type::Int))?;
                let fn_id = module
                    .get_name("rpl_jit_print_int")
                    .and_then(|f| match f {
                        cranelift_module::FuncOrDataId::Func(id) => Some(id),
                        _ => None,
                    })
                    .unwrap();
                let local_fn = module.declare_func_in_func(fn_id, builder.func);
                builder.ins().call(local_fn, &[v]);
            }
            Type::Float => {
                let v = self.compile_expr(module, builder, arg, Some(&Type::Float))?;
                let fn_id = module
                    .get_name("rpl_jit_print_float")
                    .and_then(|f| match f {
                        cranelift_module::FuncOrDataId::Func(id) => Some(id),
                        _ => None,
                    })
                    .unwrap();
                let local_fn = module.declare_func_in_func(fn_id, builder.func);
                builder.ins().call(local_fn, &[v]);
            }
            Type::Bool => {
                let v = self.compile_expr(module, builder, arg, Some(&Type::Bool))?;
                let fn_id = module
                    .get_name("rpl_jit_print_bool")
                    .and_then(|f| match f {
                        cranelift_module::FuncOrDataId::Func(id) => Some(id),
                        _ => None,
                    })
                    .unwrap();
                let local_fn = module.declare_func_in_func(fn_id, builder.func);
                builder.ins().call(local_fn, &[v]);
            }
            Type::Trit => {
                let v = self.compile_expr(module, builder, arg, Some(&Type::Trit))?;
                let fn_id = module
                    .get_name("rpl_jit_print_trit")
                    .and_then(|f| match f {
                        cranelift_module::FuncOrDataId::Func(id) => Some(id),
                        _ => None,
                    })
                    .unwrap();
                let local_fn = module.declare_func_in_func(fn_id, builder.func);
                builder.ins().call(local_fn, &[v]);
            }
            _ => {
                // String or arbitrary object converted to string
                let ptr = self.compile_expr(module, builder, arg, Some(&Type::String))?;
                let len = if let Expr::Literal(Literal::String(s), _) = arg {
                    builder.ins().iconst(types::I64, s.len() as i64)
                } else {
                    builder.ins().iconst(types::I64, -1) // usize::MAX sentinelled
                };
                let fn_id = module
                    .get_name("rpl_jit_print_str")
                    .and_then(|f| match f {
                        cranelift_module::FuncOrDataId::Func(id) => Some(id),
                        _ => None,
                    })
                    .unwrap();
                let local_fn = module.declare_func_in_func(fn_id, builder.func);
                builder.ins().call(local_fn, &[ptr, len]);
            }
        }

        Ok(builder.ins().iconst(types::I64, 0))
    }

    /// Compiles struct constructor `TypeName(arg1, arg2...)`.
    fn compile_struct_constructor<M: Module>(
        &mut self,
        module: &mut M,
        builder: &mut FunctionBuilder,
        struct_name: &str,
        args: &[Expr],
    ) -> Result<Value, CodegenCraneliftError> {
        let layout = self
            .structs
            .get(struct_name)
            .cloned()
            .ok_or_else(|| CodegenCraneliftError::UndefinedSymbol {
                name: struct_name.to_string(),
            })?;

        let align_shift = layout.alignment.max(1).trailing_zeros() as u8;
        let slot_data =
            StackSlotData::new(StackSlotKind::ExplicitSlot, layout.total_size, align_shift);
        let slot = builder.create_sized_stack_slot(slot_data);
        let slot_addr = builder.ins().stack_addr(self.ptr_type, slot, 0);

        for (i, arg) in args.iter().enumerate() {
            match arg {
                Expr::NamedArg { name, value, .. } => {
                    if let Some(flayout) = layout.get_field(name) {
                        let fval = self.compile_expr(
                            module,
                            builder,
                            value,
                            Some(&flayout.rpl_type),
                        )?;
                        builder.ins().store(
                            MemFlags::trusted(),
                            fval,
                            slot_addr,
                            flayout.offset as i32,
                        );
                    }
                }
                _ => {
                    if let Some(flayout) = layout.fields.get(i) {
                        let fval = self.compile_expr(
                            module,
                            builder,
                            arg,
                            Some(&flayout.rpl_type),
                        )?;
                        builder.ins().store(
                            MemFlags::trusted(),
                            fval,
                            slot_addr,
                            flayout.offset as i32,
                        );
                    }
                }
            }
        }

        Ok(slot_addr)
    }

    /// Lowers string interpolation fragments into chained `rpl_jit_str_concat` calls.
    fn compile_string_interpolation<M: Module>(
        &mut self,
        module: &mut M,
        builder: &mut FunctionBuilder,
        fragments: &[InterpolationFragment],
    ) -> Result<Value, CodegenCraneliftError> {
        if fragments.is_empty() {
            return self.get_string_constant_ptr(module, builder, "");
        }

        let concat_fn_id = module
            .get_name("rpl_jit_str_concat")
            .and_then(|f| match f {
                cranelift_module::FuncOrDataId::Func(id) => Some(id),
                _ => None,
            })
            .unwrap();

        let mut parts = Vec::new();

        for fragment in fragments {
            match fragment {
                InterpolationFragment::Literal(s) => {
                    let ptr = self.get_string_constant_ptr(module, builder, s)?;
                    let len = builder.ins().iconst(types::I64, s.len() as i64);
                    parts.push((ptr, len));
                }
                InterpolationFragment::Expr(expr) => {
                    let expr_ty = self.infer_expr_type(expr);
                    let val = self.compile_expr(module, builder, expr, Some(&expr_ty))?;
                    let (ptr, len) = self.convert_val_to_string_ptr(module, builder, val, &expr_ty)?;
                    parts.push((ptr, len));
                }
            }
        }

        let (mut acc_ptr, mut acc_len) = parts[0];

        for (next_ptr, next_len) in &parts[1..] {
            let local_concat = module.declare_func_in_func(concat_fn_id, builder.func);
            let call = builder.ins().call(
                local_concat,
                &[acc_ptr, acc_len, *next_ptr, *next_len],
            );
            acc_ptr = builder.inst_results(call)[0];
            acc_len = builder.ins().iconst(types::I64, -1);
        }

        Ok(acc_ptr)
    }

    /// Converts an arbitrary typed value to a string pointer and length for string interpolation.
    fn convert_val_to_string_ptr<M: Module>(
        &mut self,
        module: &mut M,
        builder: &mut FunctionBuilder,
        val: Value,
        ty: &Type,
    ) -> Result<(Value, Value), CodegenCraneliftError> {
        let minus_one = builder.ins().iconst(types::I64, -1);

        match ty {
            Type::Int => {
                let fn_id = module
                    .get_name("rpl_jit_int_to_str")
                    .and_then(|f| match f {
                        cranelift_module::FuncOrDataId::Func(id) => Some(id),
                        _ => None,
                    })
                    .unwrap();
                let local_fn = module.declare_func_in_func(fn_id, builder.func);
                let call = builder.ins().call(local_fn, &[val]);
                let ptr = builder.inst_results(call)[0];
                Ok((ptr, minus_one))
            }
            Type::Float => {
                let fn_id = module
                    .get_name("rpl_jit_float_to_str")
                    .and_then(|f| match f {
                        cranelift_module::FuncOrDataId::Func(id) => Some(id),
                        _ => None,
                    })
                    .unwrap();
                let local_fn = module.declare_func_in_func(fn_id, builder.func);
                let call = builder.ins().call(local_fn, &[val]);
                let ptr = builder.inst_results(call)[0];
                Ok((ptr, minus_one))
            }
            Type::Bool => {
                let fn_id = module
                    .get_name("rpl_jit_bool_to_str")
                    .and_then(|f| match f {
                        cranelift_module::FuncOrDataId::Func(id) => Some(id),
                        _ => None,
                    })
                    .unwrap();
                let local_fn = module.declare_func_in_func(fn_id, builder.func);
                let call = builder.ins().call(local_fn, &[val]);
                let ptr = builder.inst_results(call)[0];
                Ok((ptr, minus_one))
            }
            Type::Trit => {
                let fn_id = module
                    .get_name("rpl_jit_trit_to_str")
                    .and_then(|f| match f {
                        cranelift_module::FuncOrDataId::Func(id) => Some(id),
                        _ => None,
                    })
                    .unwrap();
                let local_fn = module.declare_func_in_func(fn_id, builder.func);
                let call = builder.ins().call(local_fn, &[val]);
                let ptr = builder.inst_results(call)[0];
                Ok((ptr, minus_one))
            }
            Type::String => Ok((val, minus_one)),
            _ => Ok((val, minus_one)),
        }
    }

    /// Retrieves or declares a constant null-terminated string literal in the Cranelift module.
    fn get_string_constant_ptr<M: Module>(
        &mut self,
        module: &mut M,
        builder: &mut FunctionBuilder,
        s: &str,
    ) -> Result<Value, CodegenCraneliftError> {
        let data_id = if let Some(id) = self.string_literals.get(s) {
            *id
        } else {
            let mut data_desc = DataDescription::new();
            let mut bytes = s.as_bytes().to_vec();
            bytes.push(0); // Null terminator
            data_desc.define(bytes.into_boxed_slice());

            let id = module.declare_anonymous_data(true, false)?;
            module.define_data(id, &data_desc)?;
            self.string_literals.insert(s.to_string(), id);
            id
        };

        let local_data = module.declare_data_in_func(data_id, builder.func);
        let ptr = builder.ins().symbol_value(self.ptr_type, local_data);
        Ok(ptr)
    }

    /// Infers the RPL type of an AST expression for code generation.
    pub fn infer_expr_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Int(_) => Type::Int,
                Literal::Float(_) => Type::Float,
                Literal::Bool(_) => Type::Bool,
                Literal::Trit(_) => Type::Trit,
                Literal::String(_) => Type::String,
            },
            Expr::Identifier(name, _) => {
                if let Some((_, ty)) = self.local_vars.get(name) {
                    ty.clone()
                } else {
                    Type::Int
                }
            }
            Expr::StringInterpolation { .. } => Type::String,
            Expr::StructBlockInit { name, .. } => Type::Named(name.clone()),
            Expr::Binary { left, op, right, .. } => {
                if op.is_comparison() {
                    Type::Bool
                } else if op.is_logical() {
                    let lt = self.infer_expr_type(left);
                    let rt = self.infer_expr_type(right);
                    if self.is_integer_type(&lt) && self.is_integer_type(&rt) {
                        lt
                    } else if lt == Type::Trit || rt == Type::Trit {
                        Type::Trit
                    } else {
                        Type::Bool
                    }
                } else {
                    self.infer_expr_type(left)
                }
            }
            Expr::Unary { op, expr, .. } => {
                if *op == UnaryOp::Not {
                    let ty = self.infer_expr_type(expr);
                    if self.is_integer_type(&ty) {
                        ty
                    } else if ty == Type::Trit {
                        Type::Trit
                    } else {
                        Type::Bool
                    }
                } else {
                    self.infer_expr_type(expr)
                }
            }
            Expr::Pipe { right, .. } => match right.as_ref() {
                Expr::Call { .. } => self.infer_expr_type(right),
                Expr::Identifier(name, _) => match name.as_str() {
                    "print" | "println" | "close_file" => Type::Int,
                    "read_file" | "read_line" => Type::String,
                    "write_file" | "append_file" | "write_line" => Type::Bool,
                    "open_file" => Type::File,
                    _ => {
                        if let Some(meta) = self.functions.get(name) {
                            meta.return_type.clone().unwrap_or(Type::Int)
                        } else {
                            Type::Int
                        }
                    }
                },
                _ => self.infer_expr_type(right),
            },
            Expr::Call { callee, .. } => {
                if let Expr::Identifier(fn_name, _) = callee.as_ref() {
                    match fn_name.as_str() {
                        "input" | "read_file" | "read_line" => return Type::String,
                        "write_file" | "append_file" | "write_line" => return Type::Bool,
                        "open_file" => return Type::File,
                        "close_file" => return Type::Int,
                        _ => {}
                    }
                    if let Some(meta) = self.functions.get(fn_name) {
                        return meta.return_type.clone().unwrap_or(Type::Int);
                    }
                    if self.structs.contains_key(fn_name) {
                        return Type::Named(fn_name.clone());
                    }
                }
                Type::Int
            }
            Expr::MemberAccess { target, field, .. } => {
                let target_ty = self.infer_expr_type(target);
                if let Type::Named(s_name) = target_ty {
                    if let Some(layout) = self.structs.get(&s_name) {
                        if let Some(flayout) = layout.get_field(field) {
                            return flayout.rpl_type.clone();
                        }
                    }
                }
                Type::Int
            }
            _ => Type::Int,
        }
    }

    fn is_integer_type(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::Int
                | Type::Byte
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
        )
    }
}
