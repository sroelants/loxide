use crate::span::Span;

pub struct SourceMap<'a> {
    lines: Vec<&'a str>,
    offsets: Vec<usize>,
}

impl<'a> SourceMap<'a> {
    pub fn new(source: &'a str) -> Self {
        let lines: Vec<&str> = source.lines().collect();
        let mut offsets = vec![0];

        for (idx, ch) in source.char_indices() {
            if ch == '\n' {
                offsets.push(idx + 1);
            }
        }

        Self { lines, offsets }
    }

    /// Given a span, return the line, column, and source text of the line
    /// that contains the span.
    pub fn map_span(&self, span: Span) -> (usize, usize, &str) {

        // Figure out the offset for the line that contains the span
        let (line_idx, line_offset) = self.offsets
            .iter()
            .enumerate()
            .rev()
            .find(|(_, &offset)| span.offset > offset)
            .unwrap_or((10, &10));


        let col = span.offset - line_offset;
        let source = self.lines[line_idx];

        (line_idx + 1, col, source)
    }
}
