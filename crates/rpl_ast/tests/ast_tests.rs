use rpl_ast::*;

#[test]
fn test_span_operations() {
    let s1 = Span::new(0, 5, 1, 1, 1, 6);
    assert_eq!(s1.len(), 5);
    assert!(!s1.is_empty());
    assert_eq!(format!("{s1}"), "1:1-6");

    let s2 = Span::new(10, 15, 2, 1, 2, 6);
    let combined = s1.combine(s2);
    assert_eq!(combined.start, 0);
    assert_eq!(combined.end, 15);
    assert_eq!(combined.start_line, 1);
    assert_eq!(combined.end_line, 2);
    assert_eq!(format!("{combined}"), "1:1-2:6");

    let dummy = Span::dummy();
    assert_eq!(dummy.len(), 0);
    assert!(dummy.is_empty());
}

#[test]
fn test_trit_logic_truth_tables() {
    use TritValue::*;

    // Kleene AND truth table
    assert_eq!(True & True, True);
    assert_eq!(True & Unknown, Unknown);
    assert_eq!(True & False, False);
    assert_eq!(Unknown & Unknown, Unknown);
    assert_eq!(Unknown & False, False);
    assert_eq!(False & False, False);

    // Kleene OR truth table
    assert_eq!(True | True, True);
    assert_eq!(True | Unknown, True);
    assert_eq!(True | False, True);
    assert_eq!(Unknown | Unknown, Unknown);
    assert_eq!(Unknown | False, Unknown);
    assert_eq!(False | False, False);

    // Kleene NOT truth table
    assert_eq!(!True, False);
    assert_eq!(!Unknown, Unknown);
    assert_eq!(!False, True);

    // Helpers and conversions
    assert_eq!(TritValue::from_bool(true), True);
    assert_eq!(TritValue::from_bool(false), False);
    assert_eq!(True.to_bool(), Some(true));
    assert_eq!(False.to_bool(), Some(false));
    assert_eq!(Unknown.to_bool(), None);

    assert!(True.is_true());
    assert!(False.is_false());
    assert!(Unknown.is_unknown());

    assert_eq!(format!("{True}"), "true");
    assert_eq!(format!("{False}"), "false");
    assert_eq!(format!("{Unknown}"), "unknown");
}

#[test]
fn test_literals_display() {
    assert_eq!(format!("{}", Literal::Int(42)), "42");
    assert_eq!(format!("{}", Literal::Float(2.75)), "2.75");
    assert_eq!(format!("{}", Literal::Float(10.0)), "10.0");
    assert_eq!(format!("{}", Literal::String("rpl".into())), "\"rpl\"");
    assert_eq!(format!("{}", Literal::Bool(true)), "true");
    assert_eq!(format!("{}", Literal::Trit(TritValue::Unknown)), "unknown");
}

#[test]
fn test_types_and_display() {
    let t_int = Type::Int;
    assert!(t_int.is_integer());
    assert!(t_int.is_numeric());
    assert!(t_int.is_primitive());
    assert_eq!(format!("{t_int}"), "Int");

    let t_u8 = Type::UInt8;
    assert!(t_u8.is_integer());
    assert_eq!(format!("{t_u8}"), "UInt8");

    let t_f32 = Type::Float32;
    assert!(t_f32.is_float());
    assert!(t_f32.is_numeric());
    assert_eq!(format!("{t_f32}"), "Float32");

    let t_list = Type::List(Box::new(Type::String));
    assert!(!t_list.is_primitive());
    assert_eq!(format!("{t_list}"), "List[String]");

    let t_map = Type::Map(Box::new(Type::String), Box::new(Type::Int));
    assert_eq!(format!("{t_map}"), "Map[String, Int]");

    let t_opt = Type::Option(Box::new(Type::Bool));
    assert_eq!(format!("{t_opt}"), "Option[Bool]");

    let t_res = Type::Result(Box::new(Type::Int64), Box::new(Type::String));
    assert_eq!(format!("{t_res}"), "Result[Int64, String]");

    let t_chan = Type::Channel(Box::new(Type::Byte));
    assert_eq!(format!("{t_chan}"), "Channel[Byte]");

    let t_named = Type::Named("ServerConfig".into());
    assert_eq!(format!("{t_named}"), "ServerConfig");
}

#[test]
fn test_operators_display_and_properties() {
    assert!(BinaryOp::Add.is_arithmetic());
    assert_eq!(format!("{}", BinaryOp::Add), "+");

    assert!(BinaryOp::Eq.is_comparison());
    assert_eq!(format!("{}", BinaryOp::Eq), "==");

    assert!(BinaryOp::And.is_logical());
    assert_eq!(format!("{}", BinaryOp::And), "and");

    assert!(BinaryOp::BitOr.is_bitwise());
    assert_eq!(format!("{}", BinaryOp::BitOr), "|");

    assert_eq!(format!("{}", UnaryOp::Not), "not ");
    assert_eq!(format!("{}", UnaryOp::Neg), "-");
    assert_eq!(format!("{}", UnaryOp::BitNot), "~");
}

#[test]
fn test_expr_variants_and_spans() {
    let s = Span::dummy();

    let lit = Expr::Literal(Literal::Int(10), s);
    assert_eq!(lit.span(), s);

    let id = Expr::Identifier("total".into(), s);
    assert_eq!(id.span(), s);

    let bin = Expr::Binary {
        left: Box::new(lit.clone()),
        op: BinaryOp::Add,
        right: Box::new(id.clone()),
        span: s,
    };
    assert_eq!(bin.span(), s);

    let un = Expr::Unary {
        op: UnaryOp::Not,
        expr: Box::new(lit.clone()),
        span: s,
    };
    assert_eq!(un.span(), s);

    let call = Expr::Call {
        callee: Box::new(id.clone()),
        args: vec![lit.clone()],
        span: s,
    };
    assert_eq!(call.span(), s);

    let pipe = Expr::Pipe {
        left: Box::new(lit.clone()),
        right: Box::new(id.clone()),
        span: s,
    };
    assert_eq!(pipe.span(), s);

    let member = Expr::MemberAccess {
        target: Box::new(id.clone()),
        field: "length".into(),
        span: s,
    };
    assert_eq!(member.span(), s);

    let interp = Expr::StringInterpolation {
        fragments: vec![
            InterpolationFragment::Literal("value: ".into()),
            InterpolationFragment::Expr(id),
        ],
        span: s,
    };
    assert_eq!(interp.span(), s);
}

#[test]
fn test_stmt_variants_and_spans() {
    let s = Span::dummy();
    let expr = Expr::Literal(Literal::Int(1), s);
    let block = Block::new(vec![Stmt::Expr(expr.clone())], s);

    let let_stmt = Stmt::Let {
        name: "x".into(),
        type_annot: Some(Type::Int),
        value: expr.clone(),
        span: s,
    };
    assert_eq!(let_stmt.span(), s);

    let mut_let_stmt = Stmt::MutLet {
        name: "counter".into(),
        type_annot: None,
        value: expr.clone(),
        span: s,
    };
    assert_eq!(mut_let_stmt.span(), s);

    let assign_stmt = Stmt::Assign {
        target: Expr::Identifier("counter".into(), s),
        value: expr.clone(),
        span: s,
    };
    assert_eq!(assign_stmt.span(), s);

    let fn_stmt = Stmt::FnDecl {
        name: "compute".into(),
        params: vec![Param::new("a".into(), Type::Int, s)],
        return_type: Some(Type::Int),
        body: block.clone(),
        span: s,
    };
    assert_eq!(fn_stmt.span(), s);

    let type_stmt = Stmt::TypeDecl {
        name: "Point".into(),
        fields: vec![
            Field::new("x".into(), Type::Float, s),
            Field::new("y".into(), Type::Float, s),
        ],
        span: s,
    };
    assert_eq!(type_stmt.span(), s);

    let if_stmt = Stmt::If {
        condition: expr.clone(),
        then_branch: block.clone(),
        else_branch: Some(block.clone()),
        span: s,
    };
    assert_eq!(if_stmt.span(), s);

    let match_stmt = Stmt::Match {
        subject: expr.clone(),
        cases: vec![MatchCase::new(
            Pattern::Literal(Literal::Trit(TritValue::True), s),
            block.clone(),
            s,
        )],
        span: s,
    };
    assert_eq!(match_stmt.span(), s);

    let for_stmt = Stmt::For {
        item_name: "item".into(),
        iterator: expr.clone(),
        body: block.clone(),
        span: s,
    };
    assert_eq!(for_stmt.span(), s);

    let par_for_stmt = Stmt::ParallelFor {
        item_name: "task".into(),
        iterator: expr.clone(),
        body: block.clone(),
        span: s,
    };
    assert_eq!(par_for_stmt.span(), s);

    let spawn_stmt = Stmt::Spawn {
        body: block.clone(),
        span: s,
    };
    assert_eq!(spawn_stmt.span(), s);

    let ret_stmt = Stmt::Return(Some(expr.clone()), s);
    assert_eq!(ret_stmt.span(), s);

    let expr_stmt = Stmt::Expr(expr);
    assert_eq!(expr_stmt.span(), s);
}

#[test]
fn test_block_deref_and_iter() {
    let s = Span::dummy();
    let stmts = vec![
        Stmt::Return(None, s),
        Stmt::Return(None, s),
    ];
    let block = Block::new(stmts, s);

    assert_eq!(block.len(), 2);
    assert!(!block.is_empty());
    assert_eq!(block[0], Stmt::Return(None, s));

    let count = block.iter().count();
    assert_eq!(count, 2);

    let into_count = block.into_iter().count();
    assert_eq!(into_count, 2);
}

#[test]
fn test_ast_error_display() {
    let s = Span::dummy();
    let err = AstError::InvalidLiteral {
        message: "number out of range".into(),
        span: s,
    };
    assert_eq!(
        format!("{err}"),
        "Invalid literal at 1:1: number out of range"
    );

    let val_err = AstError::ValidationError {
        message: "missing return in non-void function".into(),
        span: s,
    };
    assert_eq!(
        format!("{val_err}"),
        "AST validation error at 1:1: missing return in non-void function"
    );
}
