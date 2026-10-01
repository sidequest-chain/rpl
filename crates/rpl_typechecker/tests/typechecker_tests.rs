//! Integration test suite for rpl_typechecker.

use rpl_ast::Type;
use rpl_parser::parse_program;
use rpl_typechecker::{check_program, TypeError};

#[test]
fn test_project_spec_example1_words() {
    let source = r#"
fn sanitize_and_split(input_str: String) -> List[String]:
    let delimiters = [" ", ",", ".", "!", "?", ";", ":"]
    return input_str
        |> to_lower()
        |> split_any(delimiters)
        |> filter(token => token.length > 0)
end

fn analyze_text(raw_data: String) -> Map[String, Int]:
    let tokens = sanitize_and_split(raw_data)
    let mut frequency_map = Map[String, Int]()

    for token in tokens:
        let current_count = frequency_map.get(token, 0)
        frequency_map[token] = current_count + 1
    end

    for word, count in frequency_map:
        print "Word: $word, Count: $count"
    end

    return frequency_map
end
"#;
    let program = parse_program(source).expect("Failed to parse words.rpl");
    let result = check_program(&program);
    assert!(
        result.is_ok(),
        "Type checking failed for Example 1: words.rpl: {result:?}"
    );
}

#[test]
fn test_project_spec_example2_orders() {
    let source = r#"
type OrderLine:
    sku: String
    quantity: Int
    unit_price: Float
end

type Order:
    order_id: String
    lines: List[OrderLine]
    tax_rate: Float
    discount: Float
end

fn calculate_total(order: Order) -> Result[Float, String]:
    if order.lines.length == 0:
        return Error("EmptyOrder")
    end

    let mut subtotal = 0.0

    for line in order.lines:
        if line.quantity <= 0:
            return Error("InvalidQuantity")
        end if
        if line.unit_price < 0.0:
            return Error("NegativePrice")
        end if

        subtotal = subtotal + (line.quantity.to_float() * line.unit_price)
    end for

    let discount_amount = subtotal * order.discount
    let taxable = subtotal - discount_amount
    let total = taxable * (1.0 + order.tax_rate)

    return Ok(total)
end
"#;
    let program = parse_program(source).expect("Failed to parse orders.rpl");
    let result = check_program(&program);
    assert!(
        result.is_ok(),
        "Type checking failed for Example 2: orders.rpl: {result:?}"
    );
}

#[test]
fn test_project_spec_example3_ternary() {
    let source = r#"
type SensorReport:
    device_id: String
    is_operational: Trit
    has_alert: Trit
end

fn evaluate_sensor(report: SensorReport) -> Trit:
    return report.is_operational and not report.has_alert
end

fn describe_status(telemetry: SensorReport):
    let state = evaluate_sensor(telemetry)

    match state:
        case true:
            print "System $telemetry.device_id is fully operational."
        case false:
            print "ALERT: System $telemetry.device_id requires immediate action."
        case unknown:
            print "WARNING: System $telemetry.device_id status is indeterminate. Dispatching ping."
    end
end
"#;
    let program = parse_program(source).expect("Failed to parse ternary.rpl");
    let result = check_program(&program);
    assert!(
        result.is_ok(),
        "Type checking failed for Example 3: ternary.rpl: {result:?}"
    );
}

#[test]
fn test_project_spec_example4_parallel() {
    let source = r#"
type LogMetadata:
    source_ip: String
    severity: Int
end

fn process_logs(logs: List[LogMetadata]):
    let alert_channel = Channel[LogMetadata]()

    spawn:
        for alert in alert_channel:
            print "Background logger: Alert from $alert.source_ip, severity $alert.severity"
        end for
    end spawn

    parallel for item in logs:
        if item.severity >= 3:
            alert_channel.send(item)
        end if
    end for

    alert_channel.close()
end
"#;
    let program = parse_program(source).expect("Failed to parse parallel.rpl");
    let result = check_program(&program);
    assert!(
        result.is_ok(),
        "Type checking failed for Example 4: parallel.rpl: {result:?}"
    );
}

#[test]
fn test_project_spec_example5_binary() {
    let source = r#"
type FrameHeader:
    sync_byte: Byte
    message_type: UInt16
    payload_length: UInt32
end

fn parse_header(buffer: List[Byte]) -> FrameHeader:
    let sync = buffer[0]
    let msg_type = (buffer[1].to_uint16() << 8) | buffer[2].to_uint16()
    let length = (buffer[3].to_uint32() << 24)
        | (buffer[4].to_uint32() << 16)
        | (buffer[5].to_uint32() << 8)
        | buffer[6].to_uint32()

    return FrameHeader(
        sync_byte: sync,
        message_type: msg_type,
        payload_length: length
    )
end
"#;
    let program = parse_program(source).expect("Failed to parse binary.rpl");
    let result = check_program(&program);
    assert!(
        result.is_ok(),
        "Type checking failed for Example 5: binary.rpl: {result:?}"
    );
}

#[test]
fn test_type_mismatch_negative() {
    let source = r#"
fn test():
    let x: Int = "text"
end
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(result.is_err(), "Expected type mismatch error");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(
            e,
            TypeError::TypeMismatch {
                expected: Type::Int,
                found: Type::String,
                ..
            }
        )),
        "Expected TypeMismatch(Int, String), got: {errors:?}"
    );
}

#[test]
fn test_cannot_mutate_immutable_negative() {
    let source = r#"
fn test():
    let x = 1
    x = 2
end
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(
        result.is_err(),
        "Expected mutation error on immutable variable"
    );
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(
            e,
            TypeError::CannotMutateImmutable { name, .. } if name == "x"
        )),
        "Expected CannotMutateImmutable for x, got: {errors:?}"
    );
}

#[test]
fn test_mutable_variable_reassign_positive() {
    let source = r#"
fn test():
    let mut x = 1
    x = 2
end
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(result.is_ok(), "Mutating a let mut variable should succeed");
}

#[test]
fn test_non_exhaustive_trit_match_negative() {
    let source = r#"
fn test(state: Trit):
    match state:
        case true:
            print "true"
        case false:
            print "false"
    end
end
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(result.is_err(), "Expected non-exhaustive match error");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(
            e,
            TypeError::NonExhaustiveMatch { missing_cases, .. } if missing_cases.contains(&"unknown".to_string())
        )),
        "Expected NonExhaustiveMatch missing unknown, got: {errors:?}"
    );
}

#[test]
fn test_exhaustive_trit_match_positive() {
    let source = r#"
fn test(state: Trit):
    match state:
        case true:
            print "true"
        case false:
            print "false"
        case unknown:
            print "unknown"
    end
end
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(result.is_ok(), "Exhaustive Trit match should succeed");
}

#[test]
fn test_use_after_move_negative() {
    let source = r#"
type Data:
    tag: String
end

fn consume(d: Data):
    print d.tag
end

fn test():
    let my_data = Data(tag: "hello")
    consume(my_data)
    consume(my_data)
end
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(result.is_err(), "Expected use-after-move error");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(
            e,
            TypeError::UseOfMovedValue { name, .. } if name == "my_data"
        )),
        "Expected UseOfMovedValue for my_data, got: {errors:?}"
    );
}

#[test]
fn test_undefined_variable_and_function_negative() {
    let source = r#"
fn test():
    let a = nonexistent_var + 1
    call_missing_fn()
end
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(
        result.is_err(),
        "Expected undefined variable and function errors"
    );
    let errors = result.unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, TypeError::UndefinedVariable { .. })));
    assert!(errors
        .iter()
        .any(|e| matches!(e, TypeError::UndefinedFunction { .. })));
}

#[test]
fn test_kleene_logic_and_operators() {
    let source = r#"
fn test_logic(a: Trit, b: Bool) -> Trit:
    let r1 = a and b
    let r2 = a or b
    let r3 = not a
    return r1 and r2 and r3
end
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(
        result.is_ok(),
        "Kleene ternary logic expressions should type check to Trit"
    );
}

#[test]
fn test_io_typechecking_positive() {
    let source = r#"
let content = read_file("test.txt")
let w_ok = write_file("test.txt", "hello")
let a_ok = append_file("test.txt", "world")
let f: File = open_file("stream.log", "w")
let wl_ok = write_line(f, "line 1")
close_file(f)
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(
        result.is_ok(),
        "I/O builtins should pass type checking: {result:?}"
    );
}

#[test]
fn test_io_use_after_close_file_negative() {
    let source = r#"
let f = open_file("audit.log", "w")
write_line(f, "Starting")
close_file(f)
write_line(f, "Attempt write after close")
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(
        result.is_err(),
        "Expected UseOfMovedValue error after close_file"
    );
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(
            e,
            TypeError::UseOfMovedValue { name, .. } if name == "f"
        )),
        "Expected UseOfMovedValue for 'f', got: {errors:?}"
    );
}

#[test]
fn test_io_pipe_close_file_negative() {
    let source = r#"
let f = open_file("audit.log", "w")
f |> close_file
write_line(f, "Attempt write after piped close")
"#;
    let program = parse_program(source).expect("parse error");
    let result = check_program(&program);
    assert!(
        result.is_err(),
        "Expected UseOfMovedValue error after piped close_file"
    );
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| matches!(
            e,
            TypeError::UseOfMovedValue { name, .. } if name == "f"
        )),
        "Expected UseOfMovedValue for 'f', got: {errors:?}"
    );
}

