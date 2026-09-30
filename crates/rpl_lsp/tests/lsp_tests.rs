use rpl_lsp::{compute_diagnostics, get_hover_for_word, get_word_at_position};
use tower_lsp::lsp_types::{DiagnosticSeverity, Position};

#[test]
fn test_clean_program_has_no_diagnostics() {
    let source = r#"
fn tervita(nimi: String) -> Trit:
    println("Tere, $nimi")
    return true
end

fn main():
    let res: Trit = tervita("Maailm")
end
"#;

    let diags = compute_diagnostics(source);
    assert!(
        diags.is_empty(),
        "Expected no diagnostics for valid program, found: {:?}",
        diags
    );
}

#[test]
fn test_syntax_error_diagnostic() {
    // Missing 'end' to close 'fn'
    let source = r#"
fn vigane():
    println("Poolik kood")
"#;

    let diags = compute_diagnostics(source);
    assert_eq!(diags.len(), 1);
    let diag = &diags[0];
    assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
    assert!(
        diag.message.starts_with("[- - -]"),
        "Diagnostic message must start with '[- - -]', got: {}",
        diag.message
    );
    assert_eq!(diag.source.as_deref(), Some("rpl"));
}

#[test]
fn test_type_error_diagnostic() {
    // Type mismatch: expected Float, found String
    let source = r#"
fn test():
    let x: Float = "valed andmed"
end
"#;

    let diags = compute_diagnostics(source);
    assert_eq!(diags.len(), 1);
    let diag = &diags[0];
    assert_eq!(diag.severity, Some(DiagnosticSeverity::ERROR));
    assert!(
        diag.message.starts_with("[+ - -]"),
        "Diagnostic message must start with '[+ - -]', got: {}",
        diag.message
    );
    assert!(diag.message.contains("Type mismatch"));
}

#[test]
fn test_non_exhaustive_trit_match_diagnostic() {
    // Matching on Trit without handling 'unknown'
    let source = r#"
fn check_status(t: Trit):
    match t:
        case true: println("Jah")
        case false: println("Ei")
    end match
end
"#;

    let diags = compute_diagnostics(source);
    assert_eq!(diags.len(), 1);
    let diag = &diags[0];
    assert!(
        diag.message.starts_with("[+ - -]"),
        "Diagnostic message must start with '[+ - -]', got: {}",
        diag.message
    );
    assert!(diag.message.contains("Non-exhaustive match pattern"));
}

#[test]
fn test_get_word_at_position() {
    let source = "let temperatuur: Float = 312.5";
    // Hovering over "temperatuur" (col 6)
    let word = get_word_at_position(source, Position::new(0, 6));
    assert_eq!(word.as_deref(), Some("temperatuur"));

    // Hovering over "Float" (col 18)
    let word2 = get_word_at_position(source, Position::new(0, 18));
    assert_eq!(word2.as_deref(), Some("Float"));
}

#[test]
fn test_hover_builtins_and_ast_declarations() {
    let source = r#"
type Sensor:
    reading: Float
end

fn evaluate(s: Sensor) -> Trit:
    return true
end
"#;

    // Hover over built-in type Trit
    let hover_trit = get_hover_for_word("Trit", source);
    assert!(hover_trit.is_some());
    assert!(hover_trit.unwrap().contains("Kleene 3-valued logic"));

    // Hover over user-defined function evaluate
    let hover_fn = get_hover_for_word("evaluate", source);
    assert!(hover_fn.is_some());
    assert!(hover_fn.unwrap().contains("fn evaluate(s: Sensor) -> Trit"));

    // Hover over user-defined struct Sensor
    let hover_type = get_hover_for_word("Sensor", source);
    assert!(hover_type.is_some());
    assert!(hover_type.unwrap().contains("type Sensor:"));
}
