//! Source code span tracking for diagnostic reporting.

use std::fmt;

/// Represents a source code region with byte offsets and line/column positions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Span {
    /// Zero-based starting byte offset in the source file.
    pub start: usize,
    /// Zero-based ending byte offset (exclusive) in the source file.
    pub end: usize,
    /// One-based starting line number.
    pub start_line: usize,
    /// One-based starting column number.
    pub start_col: usize,
    /// One-based ending line number.
    pub end_line: usize,
    /// One-based ending column number.
    pub end_col: usize,
}

impl Span {
    /// Creates a new `Span` with explicit offsets and line/column coordinates.
    pub const fn new(
        start: usize,
        end: usize,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        Self {
            start,
            end,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    /// Creates a dummy span used for synthetic AST nodes and testing.
    pub const fn dummy() -> Self {
        Self {
            start: 0,
            end: 0,
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 1,
        }
    }

    /// Combines this span with another, producing an enclosing span that spans from
    /// the beginning of the earlier span to the end of the later span.
    pub fn combine(&self, other: Span) -> Self {
        let (start, start_line, start_col) = if self.start <= other.start {
            (self.start, self.start_line, self.start_col)
        } else {
            (other.start, other.start_line, other.start_col)
        };

        let (end, end_line, end_col) = if self.end >= other.end {
            (self.end, self.end_line, self.end_col)
        } else {
            (other.end, other.end_line, other.end_col)
        };

        Self {
            start,
            end,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    /// Returns the byte length of the span.
    pub const fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Returns `true` if the span covers zero bytes.
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start_line == self.end_line {
            if self.start_col == self.end_col {
                write!(f, "{}:{}", self.start_line, self.start_col)
            } else {
                write!(f, "{}:{}-{}", self.start_line, self.start_col, self.end_col)
            }
        } else {
            write!(
                f,
                "{}:{}-{}:{}",
                self.start_line, self.start_col, self.end_line, self.end_col
            )
        }
    }
}
