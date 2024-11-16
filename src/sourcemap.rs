use crate::span::{Span, Spanned, Annotated};

pub struct Source<'a> {
    pub source: &'a str,
    lines: Vec<&'a str>,
    offsets: Vec<usize>,
}

impl<'a> Source<'a> {
    pub fn new(source: &'a str) -> Self {
        let lines: Vec<&str> = source.lines().collect();
        let mut offsets = vec![0];

        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                offsets.push(idx + 1);
            }
        }

        Self { source, lines, offsets }
    }

    /// Given a span, return the line, column, and source text of the line
    /// that contains the span.
    pub fn map_span(&self, span: Span) -> (usize, usize, &str) {

        // Figure out the offset for the line that contains the span
        let (line_idx, line_offset) = self.offsets
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &offset)| span.offset >= offset)
            .unwrap_or((10, &10));


        let col = span.offset - line_offset;
        let source = self.lines[line_idx];

        (line_idx + 1, col, source)
    }

    pub fn annotate<T>(&self, spanned: Spanned<T>) -> Annotated<T> {
        let (line, col, source) = self.map_span(spanned.span);
        Annotated { value: spanned.value, span: spanned.span, line, col, source }
    }
}
