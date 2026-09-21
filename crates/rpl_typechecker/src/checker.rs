//! Semantic type checker and Kleene ternary logic validator for RPL AST.

use rpl_ast::{
    BinaryOp, Expr, Field, Literal, MatchCase, Pattern, Program, Span, Stmt, Type, UnaryOp,
};

use crate::env::Environment;
use crate::error::TypeError;

/// Type checker managing environment scopes, Kleene logic rules, and diagnostics.
pub struct TypeChecker {
    pub env: Environment,
    pub errors: Vec<TypeError>,
    current_return_type: Option<Type>,
}

impl Default for TypeChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeChecker {
    /// Creates a new `TypeChecker` with built-in functions and types seeded.
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            errors: Vec::new(),
            current_return_type: None,
        }
    }

    /// Performs type checking across all declarations and statements in a program.
    pub fn check_program(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        // First pass: collect top-level type and function declarations for forward referencing
        for stmt in &program.statements {
            match stmt {
                Stmt::TypeDecl { name, fields, .. } => {
                    self.env.define_type(name.clone(), fields.clone());
                }
                Stmt::FnDecl {
                    name,
                    params,
                    return_type,
                    ..
                } => {
                    self.env
                        .define_fn(name.clone(), params.clone(), return_type.clone());
                }
                _ => {}
            }
        }

        // Second pass: type check all statements
        for stmt in &program.statements {
            self.check_stmt(stmt);
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Type checks a single statement or declaration.
    pub fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let {
                name,
                type_annot,
                value,
                span,
            } => {
                let val_ty = self.check_expr(value);
                let final_ty = if let Some(expected_ty) = type_annot {
                    if !self.types_compatible(expected_ty, &val_ty) {
                        self.errors.push(TypeError::TypeMismatch {
                            expected: expected_ty.clone(),
                            found: val_ty.clone(),
                            span: value.span(),
                        });
                    }
                    expected_ty.clone()
                } else {
                    val_ty
                };

                self.env.define_var(name.clone(), final_ty, false, *span);
            }

            Stmt::MutLet {
                name,
                type_annot,
                value,
                span,
            } => {
                let val_ty = self.check_expr(value);
                let final_ty = if let Some(expected_ty) = type_annot {
                    if !self.types_compatible(expected_ty, &val_ty) {
                        self.errors.push(TypeError::TypeMismatch {
                            expected: expected_ty.clone(),
                            found: val_ty.clone(),
                            span: value.span(),
                        });
                    }
                    expected_ty.clone()
                } else {
                    val_ty
                };

                self.env.define_var(name.clone(), final_ty, true, *span);
            }

            Stmt::Assign {
                target,
                value,
                span: _,
            } => {
                let val_ty = self.check_expr(value);

                match target {
                    Expr::Identifier(name, id_span) => {
                        if let Some(binding) = self.env.lookup_var(name).cloned() {
                            if !binding.is_mut {
                                self.errors.push(TypeError::CannotMutateImmutable {
                                    name: name.clone(),
                                    span: *id_span,
                                });
                            }
                            if !self.types_compatible(&binding.ty, &val_ty) {
                                self.errors.push(TypeError::TypeMismatch {
                                    expected: binding.ty.clone(),
                                    found: val_ty,
                                    span: value.span(),
                                });
                            }
                            // Reassignment resets moved status
                            self.env.reset_moved(name);
                        } else {
                            self.errors.push(TypeError::UndefinedVariable {
                                name: name.clone(),
                                span: *id_span,
                            });
                        }
                    }
                    Expr::MemberAccess {
                        target: base,
                        field: _,
                        span: _,
                    } => {
                        let _base_ty = self.check_expr(base);
                    }
                    Expr::Index {
                        target: base,
                        index,
                        span: _,
                    } => {
                        let _base_ty = self.check_expr(base);
                        let _idx_ty = self.check_expr(index);
                    }
                    _ => {}
                }
            }

            Stmt::FnDecl {
                params,
                return_type,
                body,
                ..
            } => {
                self.env.enter_scope();
                let old_return = self.current_return_type.take();
                self.current_return_type = return_type.clone();

                for param in params {
                    self.env.define_var(
                        param.name.clone(),
                        param.param_type.clone(),
                        false,
                        param.span,
                    );
                }

                for body_stmt in &body.stmts {
                    self.check_stmt(body_stmt);
                }

                self.current_return_type = old_return;
                self.env.exit_scope();
            }

            Stmt::TypeDecl { .. } => {
                // Handled in forward-declaration pass
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let cond_ty = self.check_expr(condition);
                if cond_ty != Type::Bool && cond_ty != Type::Trit {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::Bool,
                        found: cond_ty,
                        span: condition.span(),
                    });
                }

                self.env.enter_scope();
                for s in &then_branch.stmts {
                    self.check_stmt(s);
                }
                self.env.exit_scope();

                if let Some(else_b) = else_branch {
                    self.env.enter_scope();
                    for s in &else_b.stmts {
                        self.check_stmt(s);
                    }
                    self.env.exit_scope();
                }
            }

            Stmt::Match {
                subject,
                cases,
                span,
            } => {
                let subj_ty = self.check_expr(subject);
                self.check_match_exhaustiveness(&subj_ty, cases, *span);

                for case in cases {
                    self.env.enter_scope();
                    self.bind_pattern(&case.pattern, &subj_ty);
                    if let Some(guard) = &case.guard {
                        let guard_ty = self.check_expr(guard);
                        if guard_ty != Type::Bool && guard_ty != Type::Trit {
                            self.errors.push(TypeError::TypeMismatch {
                                expected: Type::Bool,
                                found: guard_ty,
                                span: guard.span(),
                            });
                        }
                    }
                    for s in &case.body.stmts {
                        self.check_stmt(s);
                    }
                    self.env.exit_scope();
                }
            }

            Stmt::For {
                item_name,
                iterator,
                body,
                ..
            } => {
                let iter_ty = self.check_expr(iterator);
                let elem_ty = self.extract_iterator_element_type(&iter_ty, iterator.span());

                self.env.enter_scope();
                // Check if loop unpacks tuple or key, value e.g. `word, count in frequency_map`
                if item_name.contains(',') {
                    let vars: Vec<&str> = item_name.split(',').map(|s| s.trim()).collect();
                    if let Type::Map(k, v) = &elem_ty {
                        if vars.len() == 2 {
                            self.env
                                .define_var(vars[0].to_string(), *k.clone(), false, body.span);
                            self.env
                                .define_var(vars[1].to_string(), *v.clone(), false, body.span);
                        }
                    } else {
                        for v in vars {
                            self.env
                                .define_var(v.to_string(), elem_ty.clone(), false, body.span);
                        }
                    }
                } else {
                    self.env
                        .define_var(item_name.clone(), elem_ty, false, body.span);
                }

                for s in &body.stmts {
                    self.check_stmt(s);
                }
                self.env.exit_scope();
            }

            Stmt::ParallelFor {
                item_name,
                iterator,
                body,
                ..
            } => {
                let iter_ty = self.check_expr(iterator);
                let elem_ty = self.extract_iterator_element_type(&iter_ty, iterator.span());

                self.env.enter_scope();
                self.env
                    .define_var(item_name.clone(), elem_ty, false, body.span);
                for s in &body.stmts {
                    self.check_stmt(s);
                }
                self.env.exit_scope();
            }

            Stmt::Spawn { body, .. } => {
                self.env.enter_scope();
                for s in &body.stmts {
                    self.check_stmt(s);
                }
                self.env.exit_scope();
            }

            Stmt::Return(expr_opt, span) => {
                let return_ty = if let Some(expr) = expr_opt {
                    self.check_expr(expr)
                } else {
                    Type::Named("Unit".into())
                };

                if let Some(expected_ret) = &self.current_return_type {
                    if !self.types_compatible(expected_ret, &return_ty) {
                        self.errors.push(TypeError::TypeMismatch {
                            expected: expected_ret.clone(),
                            found: return_ty,
                            span: *span,
                        });
                    }
                }
            }

            Stmt::Expr(expr) => {
                self.check_expr(expr);
            }
        }
    }

    /// Recursively checks the type of an expression.
    pub fn check_expr(&mut self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Int(_) => Type::Int,
                Literal::Float(_) => Type::Float,
                Literal::Bool(_) => Type::Bool,
                Literal::Trit(_) => Type::Trit,
                Literal::String(_) => Type::String,
            },

            Expr::Identifier(name, span) => {
                if let Some(binding) = self.env.lookup_var(name) {
                    if binding.is_moved {
                        self.errors.push(TypeError::UseOfMovedValue {
                            name: name.clone(),
                            span: *span,
                        });
                    }
                    return binding.ty.clone();
                }

                // Might be a user-defined record type or generic constructor (e.g. Map, Channel, Ok)
                if self.env.lookup_type(name).is_some() || self.env.lookup_fn(name).is_some() {
                    return Type::Named(name.clone());
                }

                // Check built-in names
                if matches!(name.as_str(), "Map" | "Channel" | "List" | "Ok" | "Error") {
                    return Type::Named(name.clone());
                }

                self.errors.push(TypeError::UndefinedVariable {
                    name: name.clone(),
                    span: *span,
                });
                Type::Named("unknown".into())
            }

            Expr::Unary { op, expr, span } => {
                let target_ty = self.check_expr(expr);
                match op {
                    UnaryOp::Not => {
                        if target_ty == Type::Bool {
                            Type::Bool
                        } else if target_ty == Type::Trit {
                            Type::Trit
                        } else {
                            self.errors.push(TypeError::InvalidUnaryOp {
                                op: *op,
                                target: target_ty,
                                span: *span,
                            });
                            Type::Bool
                        }
                    }
                    UnaryOp::Neg => {
                        if matches!(
                            target_ty,
                            Type::Int
                                | Type::Float
                                | Type::Int8
                                | Type::Int16
                                | Type::Int32
                                | Type::Int64
                                | Type::Float32
                        ) {
                            target_ty
                        } else {
                            self.errors.push(TypeError::InvalidUnaryOp {
                                op: *op,
                                target: target_ty,
                                span: *span,
                            });
                            Type::Int
                        }
                    }
                    UnaryOp::BitNot => {
                        if matches!(
                            target_ty,
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
                        ) {
                            target_ty
                        } else {
                            self.errors.push(TypeError::InvalidUnaryOp {
                                op: *op,
                                target: target_ty,
                                span: *span,
                            });
                            Type::Int
                        }
                    }
                }
            }

            Expr::Binary {
                left,
                op,
                right,
                span: _,
            } => {
                let l_ty = self.check_expr(left);
                let r_ty = self.check_expr(right);

                match op {
                    BinaryOp::Add => {
                        if l_ty == Type::String && r_ty == Type::String {
                            Type::String
                        } else if self.is_numeric(&l_ty) && self.is_numeric(&r_ty) {
                            self.unify_numeric(&l_ty, &r_ty)
                        } else {
                            self.errors.push(TypeError::InvalidBinaryOp {
                                op: *op,
                                left: l_ty,
                                right: r_ty,
                                span: left.span().combine(right.span()),
                            });
                            Type::Int
                        }
                    }
                    BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        if self.is_numeric(&l_ty) && self.is_numeric(&r_ty) {
                            self.unify_numeric(&l_ty, &r_ty)
                        } else {
                            self.errors.push(TypeError::InvalidBinaryOp {
                                op: *op,
                                left: l_ty,
                                right: r_ty,
                                span: left.span().combine(right.span()),
                            });
                            Type::Int
                        }
                    }

                    // Kleene Ternary Logic for `and` and `or`
                    BinaryOp::And | BinaryOp::Or => {
                        if l_ty == Type::Bool && r_ty == Type::Bool {
                            Type::Bool
                        } else if (l_ty == Type::Trit || l_ty == Type::Bool)
                            && (r_ty == Type::Trit || r_ty == Type::Bool)
                        {
                            Type::Trit
                        } else {
                            self.errors.push(TypeError::InvalidBinaryOp {
                                op: *op,
                                left: l_ty,
                                right: r_ty,
                                span: left.span().combine(right.span()),
                            });
                            Type::Trit
                        }
                    }

                    // Relational comparisons
                    BinaryOp::Eq | BinaryOp::NotEq => {
                        if l_ty == Type::Trit || r_ty == Type::Trit {
                            Type::Trit
                        } else {
                            Type::Bool
                        }
                    }
                    BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => Type::Bool,

                    // Bitwise operators
                    BinaryOp::BitAnd
                    | BinaryOp::BitOr
                    | BinaryOp::BitXor
                    | BinaryOp::Shl
                    | BinaryOp::Shr => {
                        if self.is_integer_or_byte(&l_ty) && self.is_integer_or_byte(&r_ty) {
                            l_ty
                        } else {
                            self.errors.push(TypeError::InvalidBinaryOp {
                                op: *op,
                                left: l_ty,
                                right: r_ty,
                                span: left.span().combine(right.span()),
                            });
                            Type::Int
                        }
                    }
                }
            }

            // Pipe operator `x |> f(y)` desugars semantically into `f(x, y)`
            Expr::Pipe { left, right, .. } => {
                let left_ty = self.check_expr(left);

                match right.as_ref() {
                    Expr::Call { callee, args, span } => {
                        self.resolve_call(callee, args, Some(left_ty), *span)
                    }
                    Expr::Identifier(fn_name, span) => {
                        let callee = Box::new(Expr::Identifier(fn_name.clone(), *span));
                        self.resolve_call(&callee, &[], Some(left_ty), *span)
                    }
                    _ => self.check_expr(right),
                }
            }

            Expr::Call { callee, args, span } => self.resolve_call(callee, args, None, *span),

            Expr::MemberAccess {
                target,
                field,
                span: _,
            } => {
                let target_ty = self.check_expr(target);
                match &target_ty {
                    Type::String => match field.as_str() {
                        "length" => Type::Int,
                        _ => Type::Named("Method".into()),
                    },
                    Type::List(_) => match field.as_str() {
                        "length" => Type::Int,
                        _ => Type::Named("Method".into()),
                    },
                    Type::Map(..) => match field.as_str() {
                        "count" => Type::Int,
                        _ => Type::Named("Method".into()),
                    },
                    Type::Named(type_name) => {
                        if let Some(fields) = self.env.lookup_type(type_name).cloned() {
                            if let Some(f) = fields.iter().find(|f| f.name == *field) {
                                return f.field_type.clone();
                            }
                        }
                        Type::Named("Any".into())
                    }
                    _ => {
                        if field == "length" {
                            Type::Int
                        } else {
                            Type::Named("Any".into())
                        }
                    }
                }
            }

            Expr::Index {
                target,
                index,
                span: _,
            } => {
                let target_ty = self.check_expr(target);
                let _index_ty = self.check_expr(index);

                match target_ty {
                    Type::List(inner) => *inner,
                    Type::Map(_, val) => *val,
                    _ => Type::Named("Any".into()),
                }
            }

            Expr::List { elements, span: _ } => {
                if let Some(first) = elements.first() {
                    let elem_ty = self.check_expr(first);
                    for elem in elements.iter().skip(1) {
                        let ty = self.check_expr(elem);
                        if !self.types_compatible(&elem_ty, &ty) {
                            self.errors.push(TypeError::TypeMismatch {
                                expected: elem_ty.clone(),
                                found: ty,
                                span: elem.span(),
                            });
                        }
                    }
                    Type::List(Box::new(elem_ty))
                } else {
                    Type::List(Box::new(Type::Named("Any".into())))
                }
            }

            Expr::Range {
                start,
                end,
                span: _,
            } => {
                let s_ty = self.check_expr(start);
                let e_ty = self.check_expr(end);
                if !self.is_integer_or_byte(&s_ty) {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::Int,
                        found: s_ty,
                        span: start.span(),
                    });
                }
                if !self.is_integer_or_byte(&e_ty) {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::Int,
                        found: e_ty,
                        span: end.span(),
                    });
                }
                Type::Named("Range".into())
            }

            Expr::StringInterpolation { fragments, span: _ } => {
                for frag in fragments {
                    if let rpl_ast::InterpolationFragment::Expr(e) = frag {
                        self.check_expr(e);
                    }
                }
                Type::String
            }

            Expr::NamedArg { value, .. } => self.check_expr(value),

            Expr::Lambda {
                params,
                body,
                span: _,
            } => {
                self.env.enter_scope();
                for p in params {
                    self.env
                        .define_var(p.clone(), Type::Named("Any".into()), false, body.span());
                }
                let body_ty = self.check_expr(body);
                self.env.exit_scope();
                Type::Named(format!("Lambda[{body_ty}]"))
            }
        }
    }

    /// Resolves function, constructor, or method calls.
    fn resolve_call(
        &mut self,
        callee: &Expr,
        args: &[Expr],
        piped_arg: Option<Type>,
        span: Span,
    ) -> Type {
        // Collect evaluated argument types
        let mut arg_types = Vec::new();
        if let Some(p_ty) = piped_arg {
            arg_types.push(p_ty);
        }
        for arg in args {
            let ty = match arg {
                Expr::NamedArg { value, .. } => self.check_expr(value),
                other => self.check_expr(other),
            };
            arg_types.push(ty);

            // Ownership move detection: passing a non-copy variable to a consumer/sink function
            if let Expr::Identifier(var_name, _) = arg {
                if let Some(binding) = self.env.lookup_var(var_name) {
                    if !self.is_copy_type(&binding.ty) {
                        let is_sink = match callee {
                            Expr::Identifier(fn_name, _) => self
                                .env
                                .lookup_fn(fn_name)
                                .map(|s| s.return_type.is_none())
                                .unwrap_or(false),
                            Expr::MemberAccess { field, .. } => field == "send",
                            _ => false,
                        };
                        if is_sink {
                            self.env.mark_moved(var_name);
                        }
                    }
                }
            }
        }

        // 1. Method calls: `object.method(args)`
        if let Expr::MemberAccess { target, field, .. } = callee {
            let target_ty = self.check_expr(target);
            match (&target_ty, field.as_str()) {
                (Type::Channel(inner), "send") => {
                    if let Some(arg_ty) = arg_types.first() {
                        if !self.types_compatible(inner, arg_ty) {
                            self.errors.push(TypeError::TypeMismatch {
                                expected: *inner.clone(),
                                found: arg_ty.clone(),
                                span,
                            });
                        }
                    }
                    return Type::Named("Unit".into());
                }
                (Type::Channel(_), "close") => {
                    return Type::Named("Unit".into());
                }
                (Type::Map(k, v), "get") => {
                    if let Some(key_ty) = arg_types.first() {
                        if !self.types_compatible(k, key_ty) {
                            self.errors.push(TypeError::TypeMismatch {
                                expected: *k.clone(),
                                found: key_ty.clone(),
                                span,
                            });
                        }
                    }
                    return *v.clone();
                }
                _ => {
                    // Check helper methods like `to_float`, `to_uint16`, etc.
                    if let Some(sig) = self.env.lookup_fn(field).cloned() {
                        return sig.return_type.unwrap_or(Type::Named("Unit".into()));
                    }
                }
            }
        }

        // 2. Direct identifier calls: functions, record constructors, generic constructors
        if let Expr::Identifier(name, callee_span) = callee {
            // Built-in Ok and Error Result constructors
            if name == "Ok" {
                let inner = arg_types
                    .first()
                    .cloned()
                    .unwrap_or(Type::Named("Any".into()));
                return Type::Result(Box::new(inner), Box::new(Type::String));
            }
            if name == "Error" {
                let err_ty = arg_types.first().cloned().unwrap_or(Type::String);
                return Type::Result(Box::new(Type::Named("Any".into())), Box::new(err_ty));
            }

            // User-defined record type constructor `Type(f1: v1, f2: v2)`
            if let Some(fields) = self.env.lookup_type(name).cloned() {
                self.check_record_constructor_args(name, &fields, args, span);
                return Type::Named(name.clone());
            }

            // Defined function
            if let Some(sig) = self.env.lookup_fn(name).cloned() {
                return sig.return_type.unwrap_or(Type::Named("Unit".into()));
            }

            self.errors.push(TypeError::UndefinedFunction {
                name: name.clone(),
                span: *callee_span,
            });
            return Type::Named("unknown".into());
        }

        // 3. Index or Generic Constructors e.g. `Map[String, Int]()` or `Channel[LogMetadata]()`
        if let Expr::Index { target, index, .. } = callee {
            if let Expr::Identifier(name, _) = target.as_ref() {
                match name.as_str() {
                    "Map" => {
                        let (k, v) = self.extract_two_types(index);
                        return Type::Map(Box::new(k), Box::new(v));
                    }
                    "Channel" => {
                        let inner = self.extract_single_type(index);
                        return Type::Channel(Box::new(inner));
                    }
                    "List" => {
                        let inner = self.extract_single_type(index);
                        return Type::List(Box::new(inner));
                    }
                    _ => {}
                }
            }
        }

        Type::Named("Any".into())
    }

    /// Validates arguments passed to a record type constructor against declared fields.
    fn check_record_constructor_args(
        &mut self,
        _type_name: &str,
        fields: &[Field],
        args: &[Expr],
        _span: Span,
    ) {
        for (i, arg) in args.iter().enumerate() {
            if let Expr::NamedArg { name, value, span } = arg {
                if let Some(field) = fields.iter().find(|f| f.name == *name) {
                    let val_ty = self.check_expr(value);
                    if !self.types_compatible(&field.field_type, &val_ty) {
                        self.errors.push(TypeError::TypeMismatch {
                            expected: field.field_type.clone(),
                            found: val_ty,
                            span: *span,
                        });
                    }
                }
            } else if let Some(field) = fields.get(i) {
                let val_ty = self.check_expr(arg);
                if !self.types_compatible(&field.field_type, &val_ty) {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: field.field_type.clone(),
                        found: val_ty,
                        span: arg.span(),
                    });
                }
            }
        }
    }

    /// Checks exhaustiveness for pattern matching on `Trit` and other types.
    fn check_match_exhaustiveness(&mut self, subj_ty: &Type, cases: &[MatchCase], span: Span) {
        if *subj_ty == Type::Trit {
            let mut has_true = false;
            let mut has_false = false;
            let mut has_unknown = false;
            let mut has_wildcard = false;

            for case in cases {
                match &case.pattern {
                    Pattern::Literal(Literal::Bool(true), _) => has_true = true,
                    Pattern::Literal(Literal::Bool(false), _) => has_false = true,
                    Pattern::Literal(Literal::Trit(rpl_ast::TritValue::Unknown), _) => {
                        has_unknown = true
                    }
                    Pattern::Wildcard(_) | Pattern::Identifier(..) => has_wildcard = true,
                    _ => {}
                }
            }

            if !has_wildcard && (!has_true || !has_false || !has_unknown) {
                let mut missing = Vec::new();
                if !has_true {
                    missing.push("true".into());
                }
                if !has_false {
                    missing.push("false".into());
                }
                if !has_unknown {
                    missing.push("unknown".into());
                }
                self.errors.push(TypeError::NonExhaustiveMatch {
                    missing_cases: missing,
                    span,
                });
            }
        }
    }

    /// Binds variables from match patterns into the current scope.
    fn bind_pattern(&mut self, pattern: &Pattern, expected_ty: &Type) {
        match pattern {
            Pattern::Identifier(name, span) => {
                if name != "_" {
                    self.env
                        .define_var(name.clone(), expected_ty.clone(), false, *span);
                }
            }
            Pattern::Constructor { name, args, span } => {
                if name == "Ok" {
                    if let Type::Result(ok_ty, _) = expected_ty {
                        if let Some(arg) = args.first() {
                            self.bind_pattern(arg, ok_ty);
                        }
                    } else if let Some(arg) = args.first() {
                        self.bind_pattern(arg, &Type::Named("Any".into()));
                    }
                } else if name == "Error" {
                    if let Type::Result(_, err_ty) = expected_ty {
                        if let Some(arg) = args.first() {
                            self.bind_pattern(arg, err_ty);
                        }
                    } else if let Some(arg) = args.first() {
                        self.bind_pattern(arg, &Type::String);
                    }
                } else {
                    for arg in args {
                        self.bind_pattern(arg, &Type::Named("Any".into()));
                    }
                }
                let _ = span;
            }
            _ => {}
        }
    }

    /// Extracts element type when iterating over a collection or channel.
    fn extract_iterator_element_type(&mut self, iter_ty: &Type, span: Span) -> Type {
        match iter_ty {
            Type::List(inner) => *inner.clone(),
            Type::Channel(inner) => *inner.clone(),
            Type::Map(k, v) => Type::Map(k.clone(), v.clone()),
            Type::Named(name) if name == "Range" => Type::Int,
            Type::Named(_) => Type::Named("Any".into()),
            other => {
                self.errors.push(TypeError::TypeMismatch {
                    expected: Type::List(Box::new(Type::Named("Iterable".into()))),
                    found: other.clone(),
                    span,
                });
                Type::Named("Any".into())
            }
        }
    }

    /// Checks if a type implements copy semantics or requires move semantics.
    fn is_copy_type(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::Int
                | Type::Float
                | Type::Bool
                | Type::Trit
                | Type::Byte
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
        )
    }

    /// Checks if two types are compatible under RPL typing rules.
    pub fn types_compatible(&self, expected: &Type, actual: &Type) -> bool {
        if expected == actual {
            return true;
        }

        // `Any` matches anything
        if matches!(expected, Type::Named(n) if n == "Any")
            || matches!(actual, Type::Named(n) if n == "Any")
        {
            return true;
        }

        // Result[T, E] variant compatibility with constructors Ok/Error
        if let (Type::Result(e_ok, e_err), Type::Result(a_ok, a_err)) = (expected, actual) {
            return self.types_compatible(e_ok, a_ok) && self.types_compatible(e_err, a_err);
        }

        // Trit and Bool are compatible in Trit context
        if *expected == Type::Trit && *actual == Type::Bool {
            return true;
        }

        // Integer subtyping / conversions
        if self.is_integer_or_byte(expected) && self.is_integer_or_byte(actual) {
            return true;
        }

        false
    }

    fn is_numeric(&self, ty: &Type) -> bool {
        matches!(
            ty,
            Type::Int
                | Type::Float
                | Type::Byte
                | Type::Int8
                | Type::Int16
                | Type::Int32
                | Type::Int64
                | Type::UInt8
                | Type::UInt16
                | Type::UInt32
                | Type::UInt64
                | Type::Float32
        )
    }

    fn is_integer_or_byte(&self, ty: &Type) -> bool {
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

    fn unify_numeric(&self, l: &Type, r: &Type) -> Type {
        if *l == Type::Float || *r == Type::Float || *l == Type::Float32 || *r == Type::Float32 {
            Type::Float
        } else {
            l.clone()
        }
    }

    fn extract_single_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Identifier(name, _) => self.name_to_type(name),
            _ => Type::Named("Any".into()),
        }
    }

    fn extract_two_types(&self, expr: &Expr) -> (Type, Type) {
        match expr {
            Expr::List { elements, .. } if elements.len() >= 2 => {
                let k = self.extract_single_type(&elements[0]);
                let v = self.extract_single_type(&elements[1]);
                (k, v)
            }
            _ => (Type::Named("Any".into()), Type::Named("Any".into())),
        }
    }

    fn name_to_type(&self, name: &str) -> Type {
        match name {
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "Trit" => Type::Trit,
            "String" => Type::String,
            "Byte" => Type::Byte,
            _ => Type::Named(name.to_string()),
        }
    }
}
