use rpl_lexer::{split_interpolation, InterpolationPart, LexerError, RplLexer, Token};

#[test]
fn test_all_28_keywords() {
    let source = "and as break case channel const continue else end \
                  false fn for if in is let match mut not or \
                  parallel return spawn true type unknown while yield";

    let lexer = RplLexer::new(source);
    let tokens: Vec<Token> = lexer.map(|res| res.unwrap().0).collect();

    let expected = vec![
        Token::And,
        Token::As,
        Token::Break,
        Token::Case,
        Token::Channel,
        Token::Const,
        Token::Continue,
        Token::Else,
        Token::End,
        Token::False,
        Token::Fn,
        Token::For,
        Token::If,
        Token::In,
        Token::Is,
        Token::Let,
        Token::Match,
        Token::Mut,
        Token::Not,
        Token::Or,
        Token::Parallel,
        Token::Return,
        Token::Spawn,
        Token::True,
        Token::Type,
        Token::Unknown,
        Token::While,
        Token::Yield,
    ];

    assert_eq!(tokens, expected);
    for tok in &tokens {
        assert!(tok.is_keyword());
    }
}

#[test]
fn test_operators_and_symbols() {
    let source = "+ - * / % == != < <= > >= & | ^ ~ << >> |> -> => = : , . .. ( ) [ ]";
    let lexer = RplLexer::new(source);
    let tokens: Vec<Token> = lexer.map(|res| res.unwrap().0).collect();

    let expected = vec![
        Token::Plus,
        Token::Minus,
        Token::Star,
        Token::Slash,
        Token::Percent,
        Token::EqEq,
        Token::NotEq,
        Token::Lt,
        Token::LtEq,
        Token::Gt,
        Token::GtEq,
        Token::Ampersand,
        Token::Pipe,
        Token::Caret,
        Token::Tilde,
        Token::Shl,
        Token::Shr,
        Token::PipeRight,
        Token::Arrow,
        Token::FatArrow,
        Token::Assign,
        Token::Colon,
        Token::Comma,
        Token::Dot,
        Token::DotDot,
        Token::LParen,
        Token::RParen,
        Token::LBracket,
        Token::RBracket,
    ];

    assert_eq!(tokens, expected);
}

#[test]
fn test_numbers_parsing_and_ranges() {
    let source = "42 1_000_000 0x1A2B 0Xff 0b1010 0B1100 12.34 0.5 1e-3 2.5E4 1..100";
    let lexer = RplLexer::new(source);
    let tokens: Vec<Token> = lexer.map(|res| res.unwrap().0).collect();

    assert_eq!(tokens[0], Token::Int(42));
    assert_eq!(tokens[1], Token::Int(1_000_000));
    assert_eq!(tokens[2], Token::HexInt(0x1A2B));
    assert_eq!(tokens[3], Token::HexInt(0xFF));
    assert_eq!(tokens[4], Token::BinaryInt(0b1010));
    assert_eq!(tokens[5], Token::BinaryInt(0b1100));
    assert_eq!(tokens[6], Token::Float(12.34));
    assert_eq!(tokens[7], Token::Float(0.5));
    assert_eq!(tokens[8], Token::Float(1e-3));
    assert_eq!(tokens[9], Token::Float(2.5e4));

    // Range 1..100 must NOT parse 1. as float
    assert_eq!(tokens[10], Token::Int(1));
    assert_eq!(tokens[11], Token::DotDot);
    assert_eq!(tokens[12], Token::Int(100));
}

#[test]
fn test_strings_escapes_and_utf8() {
    let source = r#""Tere, maailm!" "Reavahetus:\nTabulaator:\tKantsulg:\" Kaldkriips:\\ Dollar:\$""#;
    let lexer = RplLexer::new(source);
    let tokens: Vec<Token> = lexer.map(|res| res.unwrap().0).collect();

    assert_eq!(tokens[0], Token::String("Tere, maailm!".into()));
    assert_eq!(
        tokens[1],
        Token::String("Reavahetus:\nTabulaator:\tKantsulg:\" Kaldkriips:\\ Dollar:$".into())
    );

    // UTF-8 in identifiers
    let id_source = "põhiseisund mõõdetud_väärtus käivita_töö ÜhikTest";
    let id_tokens: Vec<Token> = RplLexer::new(id_source).map(|res| res.unwrap().0).collect();
    assert_eq!(
        id_tokens,
        vec![
            Token::Ident("põhiseisund".into()),
            Token::Ident("mõõdetud_väärtus".into()),
            Token::Ident("käivita_töö".into()),
            Token::Ident("ÜhikTest".into()),
        ]
    );
}

#[test]
fn test_string_interpolation_detection_and_splitting() {
    let raw = "Tere $kasutaja! Arve summa on $(kogus * hind) EUR ja sümbol on \\$10.";
    let fragments = split_interpolation(raw);

    assert_eq!(
        fragments,
        vec![
            InterpolationPart::Literal("Tere ".into()),
            InterpolationPart::Variable("kasutaja".into()),
            InterpolationPart::Literal("! Arve summa on ".into()),
            InterpolationPart::Expression("kogus * hind".into()),
            InterpolationPart::Literal(" EUR ja sümbol on $10.".into()),
        ]
    );

    // Nested parentheses in $(...)
    let nested = "Tulemus: $((a + b) * (c + d))";
    let fragments_nested = split_interpolation(nested);
    assert_eq!(
        fragments_nested,
        vec![
            InterpolationPart::Literal("Tulemus: ".into()),
            InterpolationPart::Expression("(a + b) * (c + d)".into()),
        ]
    );

    // Direct token identification
    let dollar_tokens: Vec<Token> = RplLexer::new("$muutuja $(")
        .map(|res| res.unwrap().0)
        .collect();
    assert_eq!(
        dollar_tokens,
        vec![Token::DollarIdent("muutuja".into()), Token::DollarLParen]
    );
}

#[test]
fn test_block_delimiters_and_newlines() {
    let source = "if condition:\n    let x = 1\n    let y = 2\nend";
    let tokens: Vec<(Token, rpl_ast::Span)> = RplLexer::new(source).map(|res| res.unwrap()).collect();

    let token_types: Vec<Token> = tokens.iter().map(|(t, _)| t.clone()).collect();
    assert_eq!(
        token_types,
        vec![
            Token::If,
            Token::Ident("condition".into()),
            Token::Colon,
            Token::Newline,
            Token::Let,
            Token::Ident("x".into()),
            Token::Assign,
            Token::Int(1),
            Token::Newline,
            Token::Let,
            Token::Ident("y".into()),
            Token::Assign,
            Token::Int(2),
            Token::Newline,
            Token::End,
        ]
    );

    // Newlines inside parentheses and brackets are suppressed
    let multiline_call = "foo(\n    1,\n    2\n)\nlet z = 3";
    let call_tokens: Vec<Token> = RplLexer::new(multiline_call)
        .map(|res| res.unwrap().0)
        .collect();
    assert_eq!(
        call_tokens,
        vec![
            Token::Ident("foo".into()),
            Token::LParen,
            Token::Int(1),
            Token::Comma,
            Token::Int(2),
            Token::RParen,
            Token::Newline,
            Token::Let,
            Token::Ident("z".into()),
            Token::Assign,
            Token::Int(3),
        ]
    );
}

#[test]
fn test_project_spec_example3_ternary_telemetry() {
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

    let result = RplLexer::new(code).tokenize_all();
    assert!(result.is_ok(), "Failed to tokenize Example 3: {:?}", result.err());

    let tokens = result.unwrap();
    assert!(!tokens.is_empty());

    // Ensure Trit keywords are recognized properly
    assert!(tokens.iter().any(|(t, _)| *t == Token::Type));
    assert!(tokens.iter().any(|(t, _)| *t == Token::True));
    assert!(tokens.iter().any(|(t, _)| *t == Token::False));
    assert!(tokens.iter().any(|(t, _)| *t == Token::Unknown));
    assert!(tokens.iter().any(|(t, _)| *t == Token::Match));
    assert!(tokens.iter().any(|(t, _)| *t == Token::Case));
    assert!(tokens.iter().any(|(t, _)| *t == Token::Arrow));
}

#[test]
fn test_project_spec_all_other_examples() {
    // Example 1: words.rpl
    let ex1 = r#"
fn tokenize_text(input_str: String) -> List[String]:
    let delimiters = [" ", ",", ".", "!", "?", ":", ";", "\n", "\t"]
    return input_str
        |> to_lower()
        |> split_any(delimiters)
        |> filter(token => token.length > 0)
end
"#;
    assert!(RplLexer::new(ex1).tokenize_all().is_ok());

    // Example 2: orders.rpl
    let ex2 = r#"
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

let item1 = OrderLine(sku: "SYS-A100", quantity: 3, unit_price: 12.50)
"#;
    assert!(RplLexer::new(ex2).tokenize_all().is_ok());

    // Example 4: parallel.rpl
    let ex4 = r#"
type LogMetadata:
    path: String
    line_count: Int
end

fn run_pipeline():
    let pipe = Channel[LogMetadata]()
    spawn:
        parallel for index in 1..100:
            pipe.send(index * 10)
        end
        pipe.close()
    end
end
"#;
    assert!(RplLexer::new(ex4).tokenize_all().is_ok());

    // Example 5: binary.rpl
    let ex5 = r#"
type FrameHeader:
    protocol_version: UInt8
    payload_size: UInt16
    checksum: UInt32
end

let raw_bytes: List[Byte] = [
    0x01,
    0x00, 0x40,
    0x1A, 0x2B, 0x3C, 0x4D
]
"#;
    assert!(RplLexer::new(ex5).tokenize_all().is_ok());
}

#[test]
fn test_error_handling_and_spans() {
    // 1. Semicolons are forbidden in RPL!
    let semi_src = "let a = 1;";
    let semi_err = RplLexer::new(semi_src).tokenize_all().unwrap_err();
    match semi_err {
        LexerError::UnexpectedCharacter { ch, span } => {
            assert_eq!(ch, ';');
            assert_eq!(span.start_line, 1);
            assert_eq!(span.start_col, 10);
        }
        other => panic!("Expected UnexpectedCharacter for ';', got {other:?}"),
    }

    // 2. Curly braces are forbidden in RPL!
    let brace_src = "type Point { x: Int }";
    let brace_err = RplLexer::new(brace_src).tokenize_all().unwrap_err();
    match brace_err {
        LexerError::UnexpectedCharacter { ch, span } => {
            assert_eq!(ch, '{');
            assert_eq!(span.start_line, 1);
            assert_eq!(span.start_col, 12);
        }
        other => panic!("Expected UnexpectedCharacter for '{{', got {other:?}"),
    }

    // 3. Unterminated string literal
    let unterminated_src = "let msg = \"unclosed string without end";
    let str_err = RplLexer::new(unterminated_src).tokenize_all().unwrap_err();
    match str_err {
        LexerError::UnterminatedString { span } => {
            assert_eq!(span.start_line, 1);
            assert_eq!(span.start_col, 11);
        }
        other => panic!("Expected UnterminatedString, got {other:?}"),
    }

    // 4. Invalid number literals (empty hex or binary)
    let invalid_hex = "let val = 0x";
    let hex_err = RplLexer::new(invalid_hex).tokenize_all().unwrap_err();
    match hex_err {
        LexerError::InvalidNumberLiteral { message, span } => {
            assert!(message.contains("hexadecimal") || message.contains("empty") || message.contains("Empty"));
            assert_eq!(span.start_line, 1);
        }
        other => panic!("Expected InvalidNumberLiteral for '0x', got {other:?}"),
    }

    let invalid_bin = "let b = 0b";
    let bin_err = RplLexer::new(invalid_bin).tokenize_all().unwrap_err();
    match bin_err {
        LexerError::InvalidNumberLiteral { message, span } => {
            assert!(message.contains("binary") || message.contains("empty") || message.contains("Empty"));
            assert_eq!(span.start_line, 1);
        }
        other => panic!("Expected InvalidNumberLiteral for '0b', got {other:?}"),
    }

    // 5. Integer overflow
    let overflow_src = "let huge = 999999999999999999999999999999999999";
    let overflow_err = RplLexer::new(overflow_src).tokenize_all().unwrap_err();
    match overflow_err {
        LexerError::InvalidNumberLiteral { message, span } => {
            assert!(message.contains("large") || message.contains("overflow"));
            assert_eq!(span.start_line, 1);
        }
        other => panic!("Expected InvalidNumberLiteral for overflow, got {other:?}"),
    }
}
