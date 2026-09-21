use rpl_ast::{BinaryOp, Expr, Literal, Pattern, Stmt, TritValue, UnaryOp};
use rpl_parser::{parse_expression, parse_program, ParserError};

#[test]
fn test_expression_precedence_arithmetic() {
    // 1 + 2 * 3 should be 1 + (2 * 3)
    let expr = parse_expression("1 + 2 * 3").expect("failed to parse 1 + 2 * 3");
    match expr {
        Expr::Binary {
            left,
            op: BinaryOp::Add,
            right,
            ..
        } => {
            assert_eq!(*left, Expr::Literal(Literal::Int(1), left.span()));
            match *right {
                Expr::Binary {
                    left: r_left,
                    op: BinaryOp::Mul,
                    right: r_right,
                    ..
                } => {
                    assert_eq!(*r_left, Expr::Literal(Literal::Int(2), r_left.span()));
                    assert_eq!(*r_right, Expr::Literal(Literal::Int(3), r_right.span()));
                }
                other => panic!("Expected Mul on right, got {other:?}"),
            }
        }
        other => panic!("Expected Add binary, got {other:?}"),
    }

    // (1 + 2) * 3 should be (1 + 2) * 3
    let expr_grouped = parse_expression("(1 + 2) * 3").expect("failed to parse (1 + 2) * 3");
    match expr_grouped {
        Expr::Binary {
            left,
            op: BinaryOp::Mul,
            right,
            ..
        } => {
            assert_eq!(*right, Expr::Literal(Literal::Int(3), right.span()));
            match *left {
                Expr::Binary {
                    left: l_left,
                    op: BinaryOp::Add,
                    right: l_right,
                    ..
                } => {
                    assert_eq!(*l_left, Expr::Literal(Literal::Int(1), l_left.span()));
                    assert_eq!(*l_right, Expr::Literal(Literal::Int(2), l_right.span()));
                }
                other => panic!("Expected Add on left, got {other:?}"),
            }
        }
        other => panic!("Expected Mul binary, got {other:?}"),
    }
}

#[test]
fn test_pipe_operator_and_call() {
    let expr = parse_expression("x |> f(y)").expect("failed to parse pipe");
    match expr {
        Expr::Pipe { left, right, .. } => {
            assert_eq!(*left, Expr::Identifier("x".into(), left.span()));
            match *right {
                Expr::Call { callee, args, .. } => {
                    assert_eq!(*callee, Expr::Identifier("f".into(), callee.span()));
                    assert_eq!(args.len(), 1);
                    assert_eq!(args[0], Expr::Identifier("y".into(), args[0].span()));
                }
                other => panic!("Expected Call on right, got {other:?}"),
            }
        }
        other => panic!("Expected Pipe, got {other:?}"),
    }
}

#[test]
fn test_unary_and_logical_precedence() {
    let expr = parse_expression("not a and b or c").expect("failed to parse logical");
    match expr {
        Expr::Binary {
            op: BinaryOp::Or,
            left,
            right,
            ..
        } => {
            assert_eq!(*right, Expr::Identifier("c".into(), right.span()));
            match *left {
                Expr::Binary {
                    op: BinaryOp::And,
                    left: and_left,
                    right: and_right,
                    ..
                } => {
                    assert_eq!(*and_right, Expr::Identifier("b".into(), and_right.span()));
                    match *and_left {
                        Expr::Unary {
                            op: UnaryOp::Not,
                            expr: not_expr,
                            ..
                        } => {
                            assert_eq!(
                                *not_expr,
                                Expr::Identifier("a".into(), not_expr.span())
                            );
                        }
                        other => panic!("Expected Not, got {other:?}"),
                    }
                }
                other => panic!("Expected And, got {other:?}"),
            }
        }
        other => panic!("Expected Or, got {other:?}"),
    }
}

#[test]
fn test_member_access_and_indexing() {
    let expr = parse_expression("buffer[0].field").expect("failed to parse index access");
    match expr {
        Expr::MemberAccess { target, field, .. } => {
            assert_eq!(field, "field");
            match *target {
                Expr::Index { target: base, index, .. } => {
                    assert_eq!(*base, Expr::Identifier("buffer".into(), base.span()));
                    assert_eq!(*index, Expr::Literal(Literal::Int(0), index.span()));
                }
                other => panic!("Expected Index, got {other:?}"),
            }
        }
        other => panic!("Expected MemberAccess, got {other:?}"),
    }
}

#[test]
fn test_string_interpolation() {
    let expr = parse_expression("\"Hello $name, total: $(x + 1)\"").expect("failed to parse interpolated string");
    match expr {
        Expr::StringInterpolation { fragments, .. } => {
            assert_eq!(fragments.len(), 4);
        }
        other => panic!("Expected StringInterpolation, got {other:?}"),
    }
}

#[test]
fn test_lambda_expression() {
    let expr = parse_expression("token => token.length > 0").expect("failed to parse lambda");
    match expr {
        Expr::Lambda { params, body, .. } => {
            assert_eq!(params, vec!["token"]);
            match *body {
                Expr::Binary { op: BinaryOp::Gt, .. } => {}
                other => panic!("Expected Gt in lambda body, got {other:?}"),
            }
        }
        other => panic!("Expected Lambda, got {other:?}"),
    }
}

#[test]
fn test_if_else_control_structure() {
    let src = "if x > 0:\n    y = 1\nelse:\n    y = 2\nend";
    let prog = parse_program(src).expect("failed to parse if-else");
    assert_eq!(prog.statements.len(), 1);

    match &prog.statements[0] {
        Stmt::If {
            condition,
            then_branch,
            else_branch,
            ..
        } => {
            assert!(matches!(condition, Expr::Binary { op: BinaryOp::Gt, .. }));
            assert_eq!(then_branch.stmts.len(), 1);
            assert!(else_branch.is_some());
            assert_eq!(else_branch.as_ref().unwrap().stmts.len(), 1);
        }
        other => panic!("Expected Stmt::If, got {other:?}"),
    }
}

#[test]
fn test_match_control_structure() {
    let src = "match status:\n    case true:\n        x = 1\n    case false:\n        x = 2\n    case unknown:\n        x = 3\nend";
    let prog = parse_program(src).expect("failed to parse match");
    assert_eq!(prog.statements.len(), 1);

    match &prog.statements[0] {
        Stmt::Match { subject, cases, .. } => {
            assert_eq!(*subject, Expr::Identifier("status".into(), subject.span()));
            assert_eq!(cases.len(), 3);
            assert_eq!(cases[0].pattern, Pattern::Literal(Literal::Bool(true), cases[0].pattern.span()));
            assert_eq!(cases[1].pattern, Pattern::Literal(Literal::Bool(false), cases[1].pattern.span()));
            assert_eq!(cases[2].pattern, Pattern::Literal(Literal::Trit(TritValue::Unknown), cases[2].pattern.span()));
        }
        other => panic!("Expected Stmt::Match, got {other:?}"),
    }
}

#[test]
fn test_for_and_parallel_for_loops() {
    let src = "for item in items:\n    x = item\nend\n\nparallel for idx in 1..100:\n    y = idx\nend";
    let prog = parse_program(src).expect("failed to parse loops");
    assert_eq!(prog.statements.len(), 2);

    match &prog.statements[0] {
        Stmt::For { item_name, .. } => assert_eq!(item_name, "item"),
        other => panic!("Expected Stmt::For, got {other:?}"),
    }

    match &prog.statements[1] {
        Stmt::ParallelFor { item_name, iterator, .. } => {
            assert_eq!(item_name, "idx");
            assert!(matches!(iterator, Expr::Range { .. }));
        }
        other => panic!("Expected Stmt::ParallelFor, got {other:?}"),
    }
}

#[test]
fn test_spawn_and_functions() {
    let src = "spawn:\n    run_work()\nend\n\nfn add(a: Int, b: Int) -> Int:\n    return a + b\nend";
    let prog = parse_program(src).expect("failed to parse spawn and fn");
    assert_eq!(prog.statements.len(), 2);

    assert!(matches!(&prog.statements[0], Stmt::Spawn { .. }));
    match &prog.statements[1] {
        Stmt::FnDecl { name, params, return_type, .. } => {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert!(return_type.is_some());
        }
        other => panic!("Expected Stmt::FnDecl, got {other:?}"),
    }
}

#[test]
fn test_project_spec_example1_words() {
    let code = r#"
fn tokenize_text(input_str: String) -> List[String]:
    let delimiters = [" ", ",", ".", "!", "?", ":", ";", "\n", "\t"]
    return input_str
        |> to_lower()
        |> split_any(delimiters)
        |> filter(token => token.length > 0)
end

fn run_word_frequency():
    let passage = "The quiet forest spoke in autumn tones. The night was cold."
    let tokens = tokenize_text(passage)
    
    let mut frequency_map = Map[String, Int]()

    for word in tokens:
        let current_count = frequency_map.get(word, 0)
        frequency_map[word] = current_count + 1
    end

    print "--- Word Frequency Analysis (UTF-8) ---"
    print "Character count: $passage.length"
    print "Unique tokens: $frequency_map.count"

    for word, count in frequency_map:
        print "Token '$word': $count"
    end
end

run_word_frequency()
"#;
    let prog = parse_program(code).expect("Failed to parse Example 1: words.rpl");
    assert!(!prog.statements.is_empty());
}

#[test]
fn test_project_spec_example2_orders() {
    let code = r#"
type OrderLine:
    sku: String
    quantity: Int
    unit_price: Float
end

type Order:
    id: Int
    customer_id: String
    lines: List[OrderLine]
end

fn calculate_line_total(line: OrderLine) -> Result[Float, String]:
    if line.quantity <= 0:
        return Error("Invalid quantity: $line.quantity on item $line.sku")
    end
    if line.unit_price < 0.0:
        return Error("Negative price encountered on item $line.sku")
    end

    return Ok(line.quantity.to_float() * line.unit_price)
end

fn calculate_order_total(order: Order) -> Result[Float, String]:
    let mut grand_total = 0.0

    for line in order.lines:
        match calculate_line_total(line):
            case Ok(total):
                grand_total = grand_total + total
            case Error(err):
                return Error("Order calculation failed on $order.id: $err")
        end
    end

    return Ok(grand_total)
end

let item1 = OrderLine(sku: "SYS-A100", quantity: 3, unit_price: 12.50)
let item2 = OrderLine(sku: "NET-B200", quantity: 1, unit_price: 45.00)

let batch = Order(
    id: 9001,
    customer_id: "CLIENT-849",
    lines: [item1, item2]
)

match calculate_order_total(batch):
    case Ok(total_amount):
        print "Order $batch.id total: $total_amount USD"
    case Error(failure_message):
        print "ERROR: $failure_message"
end
"#;
    let prog = parse_program(code).expect("Failed to parse Example 2: orders.rpl");
    assert!(!prog.statements.is_empty());
}

#[test]
fn test_project_spec_example3_ternary() {
    let code = r#"
type SensorReport:
    device_id: String
    temperature_nominal: Trit
    pressure_valve_open: Trit
    voltage_stable: Trit
end

fn evaluate_telemetry(report: SensorReport) -> Trit:
    return report.temperature_nominal and report.pressure_valve_open and report.voltage_stable
end

let telemetry = SensorReport(
    device_id: "TURBINE-04",
    temperature_nominal: true,
    pressure_valve_open: unknown,
    voltage_stable: true
)

let system_integrity = evaluate_telemetry(telemetry)

print "--- Telemetry Diagnostic ---"
match system_integrity:
    case true:
        print "All telemetry verified. Operation within safe margins."
    case false:
        print "CRITICAL ALARM: System limits exceeded. Initiate shutdown."
    case unknown:
        print "WARNING: Sensor telemetry incomplete ($telemetry.device_id). Requesting re-scan."
end
"#;
    let prog = parse_program(code).expect("Failed to parse Example 3: ternary.rpl");
    assert!(!prog.statements.is_empty());
}

#[test]
fn test_project_spec_example4_parallel() {
    let code = r#"
type LogMetadata:
    path: String
    line_count: Int
end

fn inspect_file(file_path: String) -> LogMetadata:
    let line_estimate = file_path.length * 150
    return LogMetadata(path: file_path, line_count: line_estimate)
end

fn run_pipeline():
    let targets = [
        "/var/log/syslog",
        "/var/log/audit.log",
        "/var/log/nginx/access.log",
        "/var/log/kernel.log"
    ]

    let results_channel = Channel[LogMetadata]()

    spawn:
        parallel for path in targets:
            let info = inspect_file(path)
            results_channel.send(info)
        end
        results_channel.close()
    end

    let mut total_lines = 0
    for metadata in results_channel:
        print "Processed: $metadata.path ($metadata.line_count lines)"
        total_lines = total_lines + metadata.line_count
    end

    print "Parallel inspection complete. Total log lines: $total_lines"
end

run_pipeline()
"#;
    let prog = parse_program(code).expect("Failed to parse Example 4: parallel.rpl");
    assert!(!prog.statements.is_empty());
}

#[test]
fn test_project_spec_example5_binary() {
    let code = r#"
type FrameHeader:
    protocol_version: UInt8
    payload_size: UInt16
    checksum: UInt32
end

fn parse_frame(buffer: List[Byte]) -> Result[FrameHeader, String]:
    if buffer.length < 7:
        return Error("Frame underflow: required 7 bytes, found $buffer.length")
    end

    let version = buffer[0]
    let size = (buffer[1].to_uint16() << 8) or buffer[2].to_uint16()
    
    let c3 = buffer[3].to_uint32() << 24
    let c2 = buffer[4].to_uint32() << 16
    let c1 = buffer[5].to_uint32() << 8
    let c0 = buffer[6].to_uint32()
    let verified_crc = c3 or c2 or c1 or c0

    let header = FrameHeader(
        protocol_version: version,
        payload_size: size,
        checksum: verified_crc
    )

    return Ok(header)
end

let raw_bytes: List[Byte] = [
    0x01,
    0x00, 0x40,
    0x1A, 0x2B, 0x3C, 0x4D
]

match parse_frame(raw_bytes):
    case Ok(header):
        print "Frame successfully validated:"
        print "- Protocol Version: $header.protocol_version"
        print "- Payload Length: $header.payload_size bytes"
        print "- CRC Checksum: $header.checksum"
end
"#;
    let prog = parse_program(code).expect("Failed to parse Example 5: binary.rpl");
    assert!(!prog.statements.is_empty());
}

#[test]
fn test_parser_error_diagnostics() {
    // 1. Unclosed block (missing 'end')
    let unclosed = "if condition:\n    let x = 1\n";
    let err = parse_program(unclosed).unwrap_err();
    match err {
        ParserError::UnclosedBlock { started_at, span } => {
            assert_eq!(started_at.start_line, 1);
            assert!(span.start_line >= 1);
        }
        other => panic!("Expected UnclosedBlock, got {other:?}"),
    }

    // 2. Unexpected token
    let unexpected = "let = 10";
    let err_tok = parse_program(unexpected).unwrap_err();
    match err_tok {
        ParserError::UnexpectedToken { expected, found, .. } => {
            assert!(expected.contains("variable name"));
            assert_eq!(found, "=");
        }
        other => panic!("Expected UnexpectedToken, got {other:?}"),
    }

    // 3. Invalid assignment target
    let invalid_target = "1 + 2 = 3";
    let err_assign = parse_program(invalid_target).unwrap_err();
    match err_assign {
        ParserError::InvalidAssignmentTarget { .. } => {}
        other => panic!("Expected InvalidAssignmentTarget, got {other:?}"),
    }

    // 4. Forbidden curly braces produce LexerError via ParserError
    let brace = "type Point { x: Int }";
    let err_brace = parse_program(brace).unwrap_err();
    assert!(matches!(err_brace, ParserError::LexerError(..)));

    // 5. Forbidden semicolons produce LexerError via ParserError
    let semi = "let a = 1;";
    let err_semi = parse_program(semi).unwrap_err();
    assert!(matches!(err_semi, ParserError::LexerError(..)));
}
