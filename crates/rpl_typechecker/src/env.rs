//! Symbol table and lexical environment for scope resolution and type checking.

use std::collections::HashMap;

use rpl_ast::{Field, Param, Span, Type};

/// Binding information for a variable in a local scope.
#[derive(Debug, Clone, PartialEq)]
pub struct VarBinding {
    /// Inferred or annotated type of the variable.
    pub ty: Type,
    /// Whether the variable was declared mutable (`mut let`).
    pub is_mut: bool,
    /// Whether ownership of this value has been moved.
    pub is_moved: bool,
    /// Span where the variable was declared.
    pub declared_at: Span,
}

/// Function or method signature.
#[derive(Debug, Clone, PartialEq)]
pub struct FnSignature {
    /// Function parameter names and types.
    pub params: Vec<Param>,
    /// Return type, or `None` if returning unit.
    pub return_type: Option<Type>,
}

/// Individual lexical scope holding variable bindings.
#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub variables: HashMap<String, VarBinding>,
}

/// Environment tracking lexical scopes, type declarations, and function signatures.
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<Scope>,
    pub functions: HashMap<String, FnSignature>,
    pub types: HashMap<String, Vec<Field>>,
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

impl Environment {
    /// Creates a new environment initialized with standard RPL built-in types and functions.
    pub fn new() -> Self {
        let mut env = Self {
            scopes: vec![Scope::default()],
            functions: HashMap::new(),
            types: HashMap::new(),
        };
        env.seed_builtins();
        env
    }

    /// Seeds built-in functions, methods, and constructors described in `PROJECT_SPEC.md`.
    fn seed_builtins(&mut self) {
        // Built-in print function: takes any printable value and returns nothing (unit)
        self.functions.insert(
            "print".to_string(),
            FnSignature {
                params: vec![Param::new("msg".to_string(), Type::String, Span::dummy())],
                return_type: None,
            },
        );

        self.functions.insert(
            "println".to_string(),
            FnSignature {
                params: vec![Param::new("msg".to_string(), Type::String, Span::dummy())],
                return_type: None,
            },
        );

        self.functions.insert(
            "rpl_trit_to_str".to_string(),
            FnSignature {
                params: vec![Param::new("t".to_string(), Type::Trit, Span::dummy())],
                return_type: Some(Type::String),
            },
        );

        self.functions.insert(
            "trit_to_str".to_string(),
            FnSignature {
                params: vec![Param::new("t".to_string(), Type::Trit, Span::dummy())],
                return_type: Some(Type::String),
            },
        );

        // String helper methods
        self.functions.insert(
            "to_lower".to_string(),
            FnSignature {
                params: vec![Param::new("s".to_string(), Type::String, Span::dummy())],
                return_type: Some(Type::String),
            },
        );

        self.functions.insert(
            "split_any".to_string(),
            FnSignature {
                params: vec![
                    Param::new("s".to_string(), Type::String, Span::dummy()),
                    Param::new("delimiters".to_string(), Type::List(Box::new(Type::String)), Span::dummy()),
                ],
                return_type: Some(Type::List(Box::new(Type::String))),
            },
        );

        self.functions.insert(
            "filter".to_string(),
            FnSignature {
                params: vec![
                    Param::new("list".to_string(), Type::List(Box::new(Type::String)), Span::dummy()),
                    Param::new("predicate".to_string(), Type::Named("Lambda".to_string()), Span::dummy()),
                ],
                return_type: Some(Type::List(Box::new(Type::String))),
            },
        );

        // Numeric cast methods
        self.functions.insert(
            "to_float".to_string(),
            FnSignature {
                params: vec![Param::new("num".to_string(), Type::Int, Span::dummy())],
                return_type: Some(Type::Float),
            },
        );

        self.functions.insert(
            "to_uint16".to_string(),
            FnSignature {
                params: vec![Param::new("byte_val".to_string(), Type::Byte, Span::dummy())],
                return_type: Some(Type::UInt16),
            },
        );

        self.functions.insert(
            "to_uint32".to_string(),
            FnSignature {
                params: vec![Param::new("byte_val".to_string(), Type::Byte, Span::dummy())],
                return_type: Some(Type::UInt32),
            },
        );
    }

    /// Enters a new nested lexical scope.
    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::default());
    }

    /// Exits the current innermost lexical scope.
    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Defines a variable in the current innermost scope.
    pub fn define_var(&mut self, name: String, ty: Type, is_mut: bool, span: Span) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.variables.insert(
                name,
                VarBinding {
                    ty,
                    is_mut,
                    is_moved: false,
                    declared_at: span,
                },
            );
        }
    }

    /// Looks up a variable by name starting from the innermost scope outwards.
    pub fn lookup_var(&self, name: &str) -> Option<&VarBinding> {
        for scope in self.scopes.iter().rev() {
            if let Some(binding) = scope.variables.get(name) {
                return Some(binding);
            }
        }
        None
    }

    /// Looks up a variable mutably to update its mutation or moved state.
    pub fn lookup_var_mut(&mut self, name: &str) -> Option<&mut VarBinding> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(binding) = scope.variables.get_mut(name) {
                return Some(binding);
            }
        }
        None
    }

    /// Marks a variable as moved.
    pub fn mark_moved(&mut self, name: &str) {
        if let Some(binding) = self.lookup_var_mut(name) {
            binding.is_moved = true;
        }
    }

    /// Unmarks a variable as moved upon reassignment.
    pub fn reset_moved(&mut self, name: &str) {
        if let Some(binding) = self.lookup_var_mut(name) {
            binding.is_moved = false;
        }
    }

    /// Defines a function signature in the environment.
    pub fn define_fn(&mut self, name: String, params: Vec<Param>, return_type: Option<Type>) {
        self.functions.insert(name, FnSignature { params, return_type });
    }

    /// Looks up a function signature by name.
    pub fn lookup_fn(&self, name: &str) -> Option<&FnSignature> {
        self.functions.get(name)
    }

    /// Defines a user-defined record type and its fields.
    pub fn define_type(&mut self, name: String, fields: Vec<Field>) {
        self.types.insert(name, fields);
    }

    /// Looks up a user-defined record type definition.
    pub fn lookup_type(&self, name: &str) -> Option<&Vec<Field>> {
        self.types.get(name)
    }
}
