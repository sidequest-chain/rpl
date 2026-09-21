//! String interpolation fragment parsing for RPL string literals.

/// A fragment within an interpolated string literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InterpolationPart {
    /// Literal string text.
    Literal(String),
    /// Variable identifier (from `$ident`).
    Variable(String),
    /// Sub-expression (from `$(expr)`).
    Expression(String),
}

/// Splits a string literal's contents into literal segments and interpolation fragments.
///
/// Handles `$ident` and `$(expr)` while respecting `\$` as an escaped literal `$`.
pub fn split_interpolation(input: &str) -> Vec<InterpolationPart> {
    let mut parts = Vec::new();
    let mut current_lit = String::new();
    let mut chars = input.char_indices().peekable();

    while let Some((_, ch)) = chars.next() {
        if ch == '\\' {
            if let Some(&(_, next_ch)) = chars.peek() {
                if next_ch == '$' {
                    // Escaped dollar: produce a literal '$'
                    chars.next();
                    current_lit.push('$');
                    continue;
                }
            }
            current_lit.push('\\');
        } else if ch == '$' {
            if let Some(&(_, next_ch)) = chars.peek() {
                if next_ch == '(' {
                    // Start of $(expr)
                    chars.next(); // consume '('
                    if !current_lit.is_empty() {
                        parts.push(InterpolationPart::Literal(std::mem::take(&mut current_lit)));
                    }

                    let mut expr_buf = String::new();
                    let mut paren_nesting = 1usize;

                    for (_, inner_ch) in chars.by_ref() {
                        if inner_ch == '(' {
                            paren_nesting += 1;
                            expr_buf.push('(');
                        } else if inner_ch == ')' {
                            paren_nesting -= 1;
                            if paren_nesting == 0 {
                                break;
                            }
                            expr_buf.push(')');
                        } else {
                            expr_buf.push(inner_ch);
                        }
                    }

                    parts.push(InterpolationPart::Expression(expr_buf));
                    continue;
                } else if next_ch.is_alphabetic() || next_ch == '_' {
                    // Start of $ident
                    if !current_lit.is_empty() {
                        parts.push(InterpolationPart::Literal(std::mem::take(&mut current_lit)));
                    }

                    let mut ident_buf = String::new();
                    while let Some(&(_, id_ch)) = chars.peek() {
                        if id_ch.is_alphanumeric() || id_ch == '_' {
                            ident_buf.push(id_ch);
                            chars.next();
                        } else {
                            break;
                        }
                    }

                    parts.push(InterpolationPart::Variable(ident_buf));
                    continue;
                }
            }
            // Lone '$' not followed by '(' or identifier
            current_lit.push('$');
        } else {
            current_lit.push(ch);
        }
    }

    if !current_lit.is_empty() {
        parts.push(InterpolationPart::Literal(current_lit));
    }

    parts
}
