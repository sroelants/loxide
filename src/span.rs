use std::ops::Range;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Span {
    pub offset: usize,
    pub len: usize,
}

impl Default for Span {
    fn default() -> Self {
        Self::new()
    }
}

impl Span {
    pub const fn new() -> Self {
        Self { offset: 0, len: 0 }
    }

    pub fn new_at(offset: usize) -> Self {
        Self { offset, len: 0 }
    }

    pub fn grow(&mut self, n: usize) {
        self.len += n;
    }

    pub fn after(old: Self) -> Self {
        Self {
            offset: old.offset + old.len,
            len: 0
        }
    }

    pub fn range(&self) -> Range<usize> {
        self.offset..(self.offset + self.len)
    }

    pub fn start(&self) -> usize {
        self.offset
    }

    pub fn end(&self) -> usize {
        self.offset + self.len
    }
}
