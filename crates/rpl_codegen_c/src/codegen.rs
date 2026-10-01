//! Core C99 transpiler implementation translating validated RPL AST to C99.

use std::collections::HashMap;

use rpl_ast::{
    BinaryOp, Block, Expr, InterpolationFragment, Literal, MatchCase, Pattern, Program, Stmt,
    TritValue, Type, UnaryOp,
};

use crate::error::CodegenError;
use crate::runtime::RPL_RUNTIME_H;
use crate::types::{to_c_return_type, to_c_type};

/// Escape special characters for valid C string literals.
fn escape_c_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            _ => out.push(c),
        }
    }
    out
}

/// Sanitize identifier to prevent collision with C reserved keywords.
fn sanitize_ident(name: &str) -> String {
    match name {
        "auto" | "const" | "double" | "float" | "int" | "short" | "struct" | "unsigned"
        | "break" | "continue" | "else" | "for" | "long" | "signed" | "switch" | "void"
        | "case" | "default" | "enum" | "goto" | "register" | "sizeof" | "typedef"
        | "volatile" | "char" | "do" | "extern" | "if" | "return" | "static" | "while" => {
            format!("rpl_{name}")
        }
        "input" | "read_file" | "write_file" | "append_file" | "open_file" | "read_line"
        | "write_line" | "close_file" => format!("rpl_{name}"),
        _ => name.to_string(),
    }
}

/// Code generation context managing buffers, types, and symbol scopes.
pub struct CGenerator {
    type_decls: Vec<String>,
    prototypes: Vec<String>,
    functions: Vec<String>,
    main_stmts: Vec<String>,
    var_types: HashMap<String, Type>,
    fn_return_types: HashMap<String, Type>,
    struct_fields: HashMap<String, Vec<(String, Type)>>,
    has_user_main: bool,
    indent_level: usize,
}

impl Default for CGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CGenerator {
    /// Creates a new CGenerator instance.
    pub fn new() -> Self {
        let mut fn_return_types = HashMap::new();
        fn_return_types.insert("rpl_trit_to_str".to_string(), Type::String);
        fn_return_types.insert("trit_to_str".to_string(), Type::String);
        fn_return_types.insert("to_lower".to_string(), Type::String);
        fn_return_types.insert("input".to_string(), Type::String);
        fn_return_types.insert("read_file".to_string(), Type::String);
        fn_return_types.insert("write_file".to_string(), Type::Bool);
        fn_return_types.insert("append_file".to_string(), Type::Bool);
        fn_return_types.insert("open_file".to_string(), Type::File);
        fn_return_types.insert("read_line".to_string(), Type::String);
        fn_return_types.insert("write_line".to_string(), Type::Bool);
        fn_return_types.insert("close_file".to_string(), Type::Named("Unit".into()));

        Self {
            type_decls: Vec::new(),
            prototypes: Vec::new(),
            functions: Vec::new(),
            main_stmts: Vec::new(),
            var_types: HashMap::new(),
            fn_return_types,
            struct_fields: HashMap::new(),
            has_user_main: false,
            indent_level: 1,
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
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

    /// Transpiles the given `Program` AST into a complete, standalone C99 source string.
    pub fn generate(&mut self, program: &Program) -> Result<String, CodegenError> {
        // Pre-pass: collect types and function signatures for forward references
        for stmt in &program.statements {
            match stmt {
                Stmt::TypeDecl { name, fields, .. } => {
                    let field_pairs: Vec<(String, Type)> = fields
                        .iter()
                        .map(|f| (f.name.clone(), f.field_type.clone()))
                        .collect();
                    self.struct_fields.insert(name.clone(), field_pairs);
                }
                Stmt::FnDecl {
                    name, return_type, ..
                } => {
                    if name == "main" {
                        self.has_user_main = true;
                    }
                    if let Some(ret) = return_type {
                        self.fn_return_types.insert(name.clone(), ret.clone());
                    }
                }
                _ => {}
            }
        }

        // Pass 1: generate type declarations
        for stmt in &program.statements {
            if let Stmt::TypeDecl { name, fields, .. } = stmt {
                self.generate_type_decl(name, fields)?;
            }
        }

        // Pass 2: generate function prototypes
        for stmt in &program.statements {
            if let Stmt::FnDecl {
                name,
                params,
                return_type,
                ..
            } = stmt
            {
                let c_fn_name = if name == "main" {
                    "rpl_user_main".to_string()
                } else {
                    sanitize_ident(name)
                };
                let ret_ty = to_c_return_type(return_type.as_ref());
                let params_str = if params.is_empty() {
                    "void".to_string()
                } else {
                    params
                        .iter()
                        .map(|p| format!("{} {}", to_c_type(&p.param_type), sanitize_ident(&p.name)))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                self.prototypes
                    .push(format!("{ret_ty} {c_fn_name}({params_str});"));
            }
        }

        // Pass 3: generate function definitions and top-level statements
        for stmt in &program.statements {
            match stmt {
                Stmt::TypeDecl { .. } => {
                    // Handled in Pass 1
                }
                Stmt::FnDecl {
                    name,
                    params,
                    return_type,
                    body,
                    ..
                } => {
                    self.generate_fn_decl(name, params, return_type.as_ref(), body)?;
                }
                _ => {
                    let c_stmt = self.generate_stmt(stmt)?;
                    if !c_stmt.trim().is_empty() {
                        self.main_stmts.push(format!("{}{}", self.indent(), c_stmt));
                    }
                }
            }
        }

        // Assemble complete C99 translation unit
        let mut out = String::new();
        out.push_str(RPL_RUNTIME_H);
        out.push('\n');

        if !self.type_decls.is_empty() {
            out.push_str("/* --- User Defined Types --- */\n");
            for td in &self.type_decls {
                out.push_str(td);
                out.push('\n');
            }
            out.push('\n');
        }

        if !self.prototypes.is_empty() {
            out.push_str("/* --- Function Prototypes --- */\n");
            for proto in &self.prototypes {
                out.push_str(proto);
                out.push('\n');
            }
            out.push('\n');
        }

        if !self.functions.is_empty() {
            out.push_str("/* --- Function Definitions --- */\n");
            for func in &self.functions {
                out.push_str(func);
                out.push('\n');
            }
        }

        out.push_str("/* --- Entry Point --- */\n");
        out.push_str("int main(int argc, char** argv) {\n");
        out.push_str("    (void)argc;\n");
        out.push_str("    (void)argv;\n");

        for stmt in &self.main_stmts {
            out.push_str(stmt);
            out.push('\n');
        }

        if self.has_user_main {
            out.push_str("    rpl_user_main();\n");
        }

        out.push_str("    return 0;\n");
        out.push_str("}\n");

        Ok(out)
    }

    fn generate_type_decl(
        &mut self,
        name: &str,
        fields: &[rpl_ast::Field],
    ) -> Result<(), CodegenError> {
        let mut s = format!("typedef struct {} {{\n", sanitize_ident(name));
        for field in fields {
            s.push_str(&format!(
                "    {} {};\n",
                to_c_type(&field.field_type),
                sanitize_ident(&field.name)
            ));
        }
        s.push_str(&format!("}} {};\n", sanitize_ident(name)));
        self.type_decls.push(s);
        Ok(())
    }

    fn generate_fn_decl(
        &mut self,
        name: &str,
        params: &[rpl_ast::Param],
        return_type: Option<&Type>,
        body: &Block,
    ) -> Result<(), CodegenError> {
        let c_fn_name = if name == "main" {
            "rpl_user_main".to_string()
        } else {
            sanitize_ident(name)
        };
        let ret_ty = to_c_return_type(return_type);
        let params_str = if params.is_empty() {
            "void".to_string()
        } else {
            params
                .iter()
                .map(|p| format!("{} {}", to_c_type(&p.param_type), sanitize_ident(&p.name)))
                .collect::<Vec<_>>()
                .join(", ")
        };

        // Save local scope
        let old_vars = self.var_types.clone();
        for p in params {
            self.var_types.insert(p.name.clone(), p.param_type.clone());
        }

        let mut func_str = format!("{ret_ty} {c_fn_name}({params_str}) {{\n");
        let old_indent = self.indent_level;
        self.indent_level = 1;

        for stmt in &body.stmts {
            let s = self.generate_stmt(stmt)?;
            if !s.trim().is_empty() {
                func_str.push_str(&format!("{}{}\n", self.indent(), s));
            }
        }

        self.indent_level = old_indent;
        self.var_types = old_vars;

        func_str.push_str("}\n");
        self.functions.push(func_str);
        Ok(())
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<String, CodegenError> {
        match stmt {
            Stmt::Let {
                name,
                type_annot,
                value,
                ..
            } => {
                let inferred_ty = self.infer_expr_type(value);
                let ty = type_annot.clone().unwrap_or(inferred_ty);
                self.var_types.insert(name.clone(), ty.clone());
                let c_val = self.generate_expr_with_expected(value, Some(&ty))?;
                Ok(format!(
                    "const {} {} = {};",
                    to_c_type(&ty),
                    sanitize_ident(name),
                    c_val
                ))
            }

            Stmt::MutLet {
                name,
                type_annot,
                value,
                ..
            } => {
                let inferred_ty = self.infer_expr_type(value);
                let ty = type_annot.clone().unwrap_or(inferred_ty);
                self.var_types.insert(name.clone(), ty.clone());
                let c_val = self.generate_expr_with_expected(value, Some(&ty))?;
                Ok(format!(
                    "{} {} = {};",
                    to_c_type(&ty),
                    sanitize_ident(name),
                    c_val
                ))
            }

            Stmt::Assign { target, value, .. } => {
                let target_ty = self.infer_expr_type(target);
                let c_target = self.generate_expr(target)?;
                let c_val = self.generate_expr_with_expected(value, Some(&target_ty))?;
                Ok(format!("{c_target} = {c_val};"))
            }

            Stmt::If {
                condition,
                then_branch,
                else_branch,
                ..
            } => {
                let c_cond = self.generate_expr(condition)?;
                let cond_str = if c_cond.starts_with('(') && c_cond.ends_with(')') {
                    &c_cond[1..c_cond.len() - 1]
                } else {
                    &c_cond
                };
                let mut out = format!("if ({cond_str}) {{\n");

                self.indent_level += 1;
                for s in &then_branch.stmts {
                    let line = self.generate_stmt(s)?;
                    out.push_str(&format!("{}{}\n", self.indent(), line));
                }
                self.indent_level -= 1;

                if let Some(else_b) = else_branch {
                    out.push_str(&format!("{}}} else {{\n", self.indent()));
                    self.indent_level += 1;
                    for s in &else_b.stmts {
                        let line = self.generate_stmt(s)?;
                        out.push_str(&format!("{}{}\n", self.indent(), line));
                    }
                    self.indent_level -= 1;
                }

                out.push_str(&format!("{}}}", self.indent()));
                Ok(out)
            }

            Stmt::Match { subject, cases, .. } => {
                let subject_ty = self.infer_expr_type(subject);
                let c_subject = self.generate_expr(subject)?;
                let mut out = format!("switch ({c_subject}) {{\n");
                self.indent_level += 1;

                for case in cases {
                    out.push_str(&self.generate_match_case(case, &subject_ty)?);
                }

                self.indent_level -= 1;
                out.push_str(&format!("{}}}", self.indent()));
                Ok(out)
            }

            Stmt::For {
                item_name,
                iterator,
                body,
                ..
            } => match iterator {
                Expr::Range { start, end, .. } => {
                    let c_start = self.generate_expr(start)?;
                    let c_end = self.generate_expr(end)?;
                    let c_item = sanitize_ident(item_name);

                    self.var_types.insert(item_name.clone(), Type::Int);

                    let mut out = format!(
                        "for (int64_t {c_item} = {c_start}; {c_item} <= {c_end}; ++{c_item}) {{\n"
                    );
                    self.indent_level += 1;
                    for s in &body.stmts {
                        let line = self.generate_stmt(s)?;
                        out.push_str(&format!("{}{}\n", self.indent(), line));
                    }
                    self.indent_level -= 1;
                    out.push_str(&format!("{}}}", self.indent()));
                    Ok(out)
                }
                _ => {
                    let c_iter = self.generate_expr(iterator)?;
                    let c_item = sanitize_ident(item_name);
                    let mut out = format!("/* Iterate over {c_iter} */ {{\n");
                    self.indent_level += 1;
                    out.push_str(&format!(
                        "{}const char* {c_item} = {c_iter};\n",
                        self.indent()
                    ));
                    for s in &body.stmts {
                        let line = self.generate_stmt(s)?;
                        out.push_str(&format!("{}{}\n", self.indent(), line));
                    }
                    self.indent_level -= 1;
                    out.push_str(&format!("{}}}", self.indent()));
                    Ok(out)
                }
            },

            Stmt::Return(opt_expr, _) => match opt_expr {
                Some(expr) => {
                    let c_expr = self.generate_expr(expr)?;
                    Ok(format!("return {c_expr};"))
                }
                None => Ok("return;".to_string()),
            },

            Stmt::Expr(expr) => {
                let c_expr = self.generate_expr(expr)?;
                Ok(format!("{c_expr};"))
            }

            Stmt::Spawn { body, .. } | Stmt::ParallelFor { body, .. } => {
                // Phase 1 fallback for concurrency: execute sequentially in C99
                let mut out = "/* sequential concurrency fallback */ {\n".to_string();
                self.indent_level += 1;
                for s in &body.stmts {
                    let line = self.generate_stmt(s)?;
                    out.push_str(&format!("{}{}\n", self.indent(), line));
                }
                self.indent_level -= 1;
                out.push_str(&format!("{}}}", self.indent()));
                Ok(out)
            }

            _ => Ok(String::new()),
        }
    }

    fn generate_match_case(
        &mut self,
        case: &MatchCase,
        subject_ty: &Type,
    ) -> Result<String, CodegenError> {
        let mut out = String::new();
        match &case.pattern {
            Pattern::Literal(Literal::Trit(trit), _) => {
                let label = match trit {
                    TritValue::True => "case RPL_TRIT_TRUE:",
                    TritValue::False => "case RPL_TRIT_FALSE:",
                    TritValue::Unknown => "case RPL_TRIT_UNKNOWN:",
                };
                out.push_str(&format!("{}{}\n", self.indent(), label));
            }
            Pattern::Literal(Literal::Bool(b), _) => {
                if *subject_ty == Type::Trit {
                    let label = if *b {
                        "case RPL_TRIT_TRUE:"
                    } else {
                        "case RPL_TRIT_FALSE:"
                    };
                    out.push_str(&format!("{}{}\n", self.indent(), label));
                } else {
                    out.push_str(&format!(
                        "{}case {}:\n",
                        self.indent(),
                        if *b { "1" } else { "0" }
                    ));
                }
            }
            Pattern::Literal(Literal::Int(v), _) => {
                out.push_str(&format!("{}case {}LL:\n", self.indent(), v));
            }
            Pattern::Wildcard(_) | Pattern::Identifier(_, _) => {
                out.push_str(&format!("{}default:\n", self.indent()));
            }
            Pattern::Constructor { name, args, .. } => {
                // For Result/Option constructors: Ok(v), Error(e)
                let c_name = sanitize_ident(name);
                out.push_str(&format!("{}/* case {c_name} */ default:\n", self.indent()));
                if let Some(Pattern::Identifier(arg_name, _)) = args.first() {
                    self.var_types.insert(arg_name.clone(), Type::String);
                }
            }
            Pattern::Literal(Literal::String(s), _) => {
                out.push_str(&format!("{}/* string match: {s} */ default:\n", self.indent()));
            }
            Pattern::Literal(Literal::Float(_), _) => {
                out.push_str(&format!("{}default:\n", self.indent()));
            }
        }

        self.indent_level += 1;
        out.push_str(&format!("{} {{\n", self.indent()));
        self.indent_level += 1;

        for s in &case.body.stmts {
            let line = self.generate_stmt(s)?;
            out.push_str(&format!("{}{}\n", self.indent(), line));
        }

        out.push_str(&format!("{}break;\n", self.indent()));
        self.indent_level -= 1;
        out.push_str(&format!("{}}}\n", self.indent()));
        self.indent_level -= 1;

        Ok(out)
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<String, CodegenError> {
        self.generate_expr_with_expected(expr, None)
    }

    fn generate_expr_with_expected(
        &mut self,
        expr: &Expr,
        expected_type: Option<&Type>,
    ) -> Result<String, CodegenError> {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Int(v) => Ok(format!("{v}LL")),
                Literal::Float(v) => {
                    if v.fract() == 0.0 {
                        Ok(format!("{v:.1}"))
                    } else {
                        Ok(format!("{v}"))
                    }
                }
                Literal::String(s) => Ok(format!("\"{}\"", escape_c_string(s))),
                Literal::Bool(b) => {
                    if expected_type == Some(&Type::Trit) {
                        Ok(if *b {
                            "RPL_TRIT_TRUE".into()
                        } else {
                            "RPL_TRIT_FALSE".into()
                        })
                    } else {
                        Ok(if *b { "true".into() } else { "false".into() })
                    }
                }
                Literal::Trit(t) => match t {
                    TritValue::True => Ok("RPL_TRIT_TRUE".into()),
                    TritValue::False => Ok("RPL_TRIT_FALSE".into()),
                    TritValue::Unknown => Ok("RPL_TRIT_UNKNOWN".into()),
                },
            },

            Expr::Identifier(name, _) => Ok(sanitize_ident(name)),

            Expr::Binary {
                left, op, right, ..
            } => {
                let left_ty = self.infer_expr_type(left);
                let right_ty = self.infer_expr_type(right);
                let is_trit = expected_type == Some(&Type::Trit)
                    || left_ty == Type::Trit
                    || right_ty == Type::Trit;

                let op_expected = if is_trit { Some(&Type::Trit) } else { None };
                let c_left = self.generate_expr_with_expected(left, op_expected)?;
                let c_right = self.generate_expr_with_expected(right, op_expected)?;

                match op {
                    BinaryOp::Add => Ok(format!("({c_left} + {c_right})")),
                    BinaryOp::Sub => Ok(format!("({c_left} - {c_right})")),
                    BinaryOp::Mul => Ok(format!("({c_left} * {c_right})")),
                    BinaryOp::Div => Ok(format!("({c_left} / {c_right})")),
                    BinaryOp::Mod => Ok(format!("({c_left} % {c_right})")),

                    BinaryOp::Eq => Ok(format!("({c_left} == {c_right})")),
                    BinaryOp::NotEq => Ok(format!("({c_left} != {c_right})")),
                    BinaryOp::Lt => Ok(format!("({c_left} < {c_right})")),
                    BinaryOp::LtEq => Ok(format!("({c_left} <= {c_right})")),
                    BinaryOp::Gt => Ok(format!("({c_left} > {c_right})")),
                    BinaryOp::GtEq => Ok(format!("({c_left} >= {c_right})")),

                    BinaryOp::And => {
                        if is_trit {
                            Ok(format!("rpl_trit_and({c_left}, {c_right})"))
                        } else if self.is_integer_or_byte(&left_ty) && self.is_integer_or_byte(&right_ty) {
                            Ok(format!("({c_left} & {c_right})"))
                        } else {
                            Ok(format!("({c_left} && {c_right})"))
                        }
                    }
                    BinaryOp::Or => {
                        if is_trit {
                            Ok(format!("rpl_trit_or({c_left}, {c_right})"))
                        } else if self.is_integer_or_byte(&left_ty) && self.is_integer_or_byte(&right_ty) {
                            Ok(format!("({c_left} | {c_right})"))
                        } else {
                            Ok(format!("({c_left} || {c_right})"))
                        }
                    }

                    BinaryOp::BitAnd => Ok(format!("({c_left} & {c_right})")),
                    BinaryOp::BitOr => Ok(format!("({c_left} | {c_right})")),
                    BinaryOp::BitXor => Ok(format!("({c_left} ^ {c_right})")),
                    BinaryOp::Shl => Ok(format!("({c_left} << {c_right})")),
                    BinaryOp::Shr => Ok(format!("({c_left} >> {c_right})")),
                }
            }

            Expr::Unary { op, expr, .. } => {
                let expr_ty = self.infer_expr_type(expr);
                let is_trit = expected_type == Some(&Type::Trit) || expr_ty == Type::Trit;
                let c_expr = self.generate_expr_with_expected(
                    expr,
                    if is_trit { Some(&Type::Trit) } else { None },
                )?;

                match op {
                    UnaryOp::Not => {
                        if is_trit {
                            Ok(format!("rpl_trit_not({c_expr})"))
                        } else if self.is_integer_or_byte(&expr_ty) {
                            Ok(format!("(~{c_expr})"))
                        } else {
                            Ok(format!("(!{c_expr})"))
                        }
                    }
                    UnaryOp::Neg => Ok(format!("(-{c_expr})")),
                    UnaryOp::BitNot => Ok(format!("(~{c_expr})")),
                }
            }

            Expr::Pipe { left, right, .. } => {
                let c_left = self.generate_expr(left)?;
                match right.as_ref() {
                    Expr::Call { callee, args, .. } => {
                        let c_callee = self.generate_expr(callee)?;
                        let mut all_args = vec![c_left];
                        for a in args {
                            all_args.push(self.generate_expr(a)?);
                        }
                        Ok(format!("{}({})", c_callee, all_args.join(", ")))
                    }
                    _ => {
                        let c_right = self.generate_expr(right)?;
                        Ok(format!("{c_right}({c_left})"))
                    }
                }
            }

            Expr::Call { callee, args, .. } => {
                if let Expr::Identifier(fn_name, _) = callee.as_ref() {
                    if fn_name == "print" || fn_name == "println" {
                        return self.generate_print_call(fn_name == "println", args);
                    }

                    // Check if it's a struct constructor: TypeName(...)
                    if self.struct_fields.contains_key(fn_name) {
                        return self.generate_struct_constructor(fn_name, args);
                    }

                    // Special builtins
                    if fn_name == "Ok" {
                        if let Some(first) = args.first() {
                            return self.generate_expr(first);
                        }
                        return Ok("0".to_string());
                    }
                    if fn_name == "Error" {
                        if let Some(first) = args.first() {
                            return self.generate_expr(first);
                        }
                        return Ok("NULL".to_string());
                    }
                }

                let c_callee = self.generate_expr(callee)?;
                let mut c_args = Vec::new();
                for a in args {
                    c_args.push(self.generate_expr(a)?);
                }
                Ok(format!("{}({})", c_callee, c_args.join(", ")))
            }

            Expr::MemberAccess { target, field, .. } => {
                let c_target = self.generate_expr(target)?;
                let c_field = sanitize_ident(field);

                // Handle special method calls like x.to_float() or x.length
                if c_field == "to_float" {
                    return Ok(format!("((double)({c_target}))"));
                }
                if c_field == "length" || c_field == "count" {
                    return Ok(format!("strlen({c_target})"));
                }

                Ok(format!("{c_target}.{c_field}"))
            }

            Expr::Index { target, index, .. } => {
                let c_target = self.generate_expr(target)?;
                let c_index = self.generate_expr(index)?;
                Ok(format!("{c_target}[{c_index}]"))
            }

            Expr::List { elements, .. } => {
                let mut c_elems = Vec::new();
                for elem in elements {
                    c_elems.push(self.generate_expr(elem)?);
                }
                Ok(format!("{{ {} }}", c_elems.join(", ")))
            }

            Expr::StringInterpolation { fragments, .. } => {
                self.generate_string_interpolation(fragments)
            }

            Expr::Range { start, end, .. } => {
                let c_start = self.generate_expr(start)?;
                let c_end = self.generate_expr(end)?;
                Ok(format!("{c_start}..{c_end}"))
            }

            Expr::NamedArg { name, value, .. } => {
                let c_val = self.generate_expr(value)?;
                Ok(format!(".{} = {}", sanitize_ident(name), c_val))
            }

            Expr::Lambda { params, body, .. } => {
                // In C99, represent lambda inline or evaluate body
                let c_body = self.generate_expr(body)?;
                Ok(format!("/* lambda({}) */ ({c_body})", params.join(", ")))
            }

            Expr::StructBlockInit { name, fields, .. } => {
                let struct_def_fields = self
                    .struct_fields
                    .get(name)
                    .cloned()
                    .unwrap_or_default();
                let mut field_inits = Vec::new();
                for (fname, fval) in fields {
                    let field_ty = struct_def_fields
                        .iter()
                        .find(|(n, _)| n == fname)
                        .map(|(_, t)| t);
                    let c_val = self.generate_expr_with_expected(fval, field_ty)?;
                    field_inits.push(format!(".{} = {}", sanitize_ident(fname), c_val));
                }
                Ok(format!("({}){{ {} }}", sanitize_ident(name), field_inits.join(", ")))
            }
        }
    }

    fn generate_print_call(
        &mut self,
        is_println: bool,
        args: &[Expr],
    ) -> Result<String, CodegenError> {
        if args.is_empty() {
            return Ok(if is_println {
                "println(\"\")".to_string()
            } else {
                "print(\"\")".to_string()
            });
        }

        if args.len() == 1 {
            let arg = &args[0];
            let arg_ty = self.infer_expr_type(arg);
            let c_arg = self.generate_expr(arg)?;

            match arg_ty {
                Type::Int => {
                    return Ok(format!("printf(\"%\" PRId64 \"\\n\", {c_arg})"));
                }
                Type::Float => {
                    return Ok(format!("printf(\"%g\\n\", {c_arg})"));
                }
                Type::Bool => {
                    return Ok(format!("printf(\"%s\\n\", ({c_arg}) ? \"true\" : \"false\")"));
                }
                Type::Trit => {
                    return Ok(format!("printf(\"%s\\n\", rpl_trit_to_str({c_arg}))"));
                }
                _ => {
                    let fn_name = if is_println { "println" } else { "print" };
                    return Ok(format!("{fn_name}({c_arg})"));
                }
            }
        }

        let mut c_args = Vec::new();
        for a in args {
            c_args.push(self.generate_expr(a)?);
        }
        let fn_name = if is_println { "println" } else { "print" };
        Ok(format!("{fn_name}({})", c_args.join(", ")))
    }

    fn generate_struct_constructor(
        &mut self,
        struct_name: &str,
        args: &[Expr],
    ) -> Result<String, CodegenError> {
        let c_struct = sanitize_ident(struct_name);
        let fields = self
            .struct_fields
            .get(struct_name)
            .cloned()
            .unwrap_or_default();
        let mut field_inits = Vec::new();

        for (i, arg) in args.iter().enumerate() {
            match arg {
                Expr::NamedArg { name, value, .. } => {
                    let field_ty = fields.iter().find(|(n, _)| n == name).map(|(_, t)| t);
                    let c_val = self.generate_expr_with_expected(value, field_ty)?;
                    field_inits.push(format!(".{} = {}", sanitize_ident(name), c_val));
                }
                _ => {
                    let field_ty = fields.get(i).map(|(_, t)| t);
                    let c_val = self.generate_expr_with_expected(arg, field_ty)?;
                    field_inits.push(c_val);
                }
            }
        }

        Ok(format!("({c_struct}){{ {} }}", field_inits.join(", ")))
    }

    fn generate_string_interpolation(
        &mut self,
        fragments: &[InterpolationFragment],
    ) -> Result<String, CodegenError> {
        if fragments.is_empty() {
            return Ok("\"\"".to_string());
        }

        let mut parts = Vec::new();
        for fragment in fragments {
            match fragment {
                InterpolationFragment::Literal(s) => {
                    parts.push(format!("\"{}\"", escape_c_string(s)));
                }
                InterpolationFragment::Expr(expr) => {
                    let ty = self.infer_expr_type(expr);
                    let c_expr = self.generate_expr(expr)?;
                    match ty {
                        Type::Int => parts.push(format!("rpl_int_to_str({c_expr})")),
                        Type::Float => parts.push(format!("rpl_float_to_str({c_expr})")),
                        Type::Bool => parts.push(format!("rpl_bool_to_str({c_expr})")),
                        Type::Trit => parts.push(format!("rpl_trit_to_str({c_expr})")),
                        _ => parts.push(c_expr),
                    }
                }
            }
        }

        if parts.len() == 1 {
            return Ok(parts[0].clone());
        }

        // Chain with rpl_str_concat(a, b)
        let mut result = parts[0].clone();
        for part in &parts[1..] {
            result = format!("rpl_str_concat({result}, {part})");
        }
        Ok(result)
    }

    fn infer_expr_type(&self, expr: &Expr) -> Type {
        match expr {
            Expr::Literal(lit, _) => match lit {
                Literal::Int(_) => Type::Int,
                Literal::Float(_) => Type::Float,
                Literal::Bool(_) => Type::Bool,
                Literal::Trit(_) => Type::Trit,
                Literal::String(_) => Type::String,
            },
            Expr::Identifier(name, _) => self.var_types.get(name).cloned().unwrap_or(Type::Int),
            Expr::StringInterpolation { .. } => Type::String,
            Expr::StructBlockInit { name, .. } => Type::Named(name.clone()),
            Expr::Binary {
                left, op, right, ..
            } => {
                if op.is_comparison() {
                    Type::Bool
                } else if op.is_logical() {
                    let lt = self.infer_expr_type(left);
                    let rt = self.infer_expr_type(right);
                    if self.is_integer_or_byte(&lt) && self.is_integer_or_byte(&rt) {
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
                    if self.is_integer_or_byte(&ty) {
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
            Expr::Call { callee, .. } => {
                if let Expr::Identifier(fn_name, _) = callee.as_ref() {
                    if let Some(ret) = self.fn_return_types.get(fn_name) {
                        return ret.clone();
                    }
                    if self.struct_fields.contains_key(fn_name) {
                        return Type::Named(fn_name.clone());
                    }
                }
                Type::Int
            }
            Expr::MemberAccess { target, field, .. } => {
                if field == "to_float" {
                    return Type::Float;
                }
                if field == "length" || field == "count" {
                    return Type::Int;
                }
                let target_ty = self.infer_expr_type(target);
                if let Type::Named(s_name) = target_ty {
                    if let Some(fields) = self.struct_fields.get(&s_name) {
                        if let Some((_, f_ty)) = fields.iter().find(|(n, _)| n == field) {
                            return f_ty.clone();
                        }
                    }
                }
                Type::Int
            }
            Expr::Pipe { right, .. } => self.infer_expr_type(right),
            _ => Type::Int,
        }
    }
}
