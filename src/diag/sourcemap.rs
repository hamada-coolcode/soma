use crate::diag::Span;

pub type FileId = usize;

#[derive(Debug, Default)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

#[derive(Debug)]
pub struct SourceFile {
    pub name: String,
    pub text: String,
}

impl SourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: impl Into<String>, text: impl Into<String>) -> FileId {
        self.files.push(SourceFile {
            name: name.into(),
            text: text.into(),
        });
        self.files.len() - 1
    }

    pub fn source(&self, id: FileId) -> &str {
        &self.files[id].text
    }

    pub fn name(&self, id: FileId) -> &str {
        &self.files[id].name
    }

    pub fn file_count(&self) -> usize {
        self.files.len()
    }

    pub fn slice(&self, id: FileId, span: Span) -> &str {
        let text = self.source(id);
        let mut start = span.start.min(text.len());
        let mut end = span.end.min(text.len());

        while start > 0 && !text.is_char_boundary(start) {
            start -= 1;
        }
        while end < text.len() && !text.is_char_boundary(end) {
            end += 1;
        }
        if start > end {
            start = end;
        }
        &text[start..end]
    }

    /// 1-based (line, column) for a byte offset.
    pub fn offset_at(&self, id: FileId, line: usize, col: usize) -> Option<usize> {
        if line == 0 || col == 0 {
            return None;
        }

        let text = self.source(id);
        let mut current_line = 1;
        let mut line_start = 0;

        for (i, b) in text.bytes().enumerate() {
            if current_line == line {
                break;
            }
            if b == b'\n' {
                current_line += 1;
                line_start = i + 1;
            }
        }

        if current_line != line {
            return None;
        }

        let target_offset = line_start + col - 1;
        if target_offset > text.len() {
            return None;
        }

        let line_end = text[line_start..]
            .find('\n')
            .map(|i| line_start + i)
            .unwrap_or(text.len());
        if target_offset > line_end {
            return None;
        }

        Some(target_offset)
    }

    pub fn line_col(&self, id: FileId, offset: usize) -> (usize, usize) {
        let text = self.source(id);
        let upto = offset.min(text.len());

        let mut line = 1;
        let mut last_newline = 0;

        for (i, b) in text[..upto].bytes().enumerate() {
            if b == b'\n' {
                line += 1;
                last_newline = i + 1;
            }
        }

        let col = upto - last_newline + 1;
        (line, col)
    }

    pub fn map_span(&self, from: FileId, to: FileId, span: Span) -> Option<Span> {
        let (start_line, start_col) = self.line_col(from, span.start);
        let (end_line, end_col) = self.line_col(from, span.end);

        let mapped_start = self.offset_at(to, start_line, start_col)?;
        let mapped_end = self.offset_at(to, end_line, end_col)?;

        Some(Span::new(mapped_start, mapped_end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_count() {
        let mut sm = SourceMap::new();
        assert_eq!(sm.file_count(), 0);
        sm.add("file1", "content");
        assert_eq!(sm.file_count(), 1);
        sm.add("file2", "content2");
        assert_eq!(sm.file_count(), 2);
    }

    #[test]
    fn test_slice_clamping() {
        let mut sm = SourceMap::new();
        let id = sm.add("test", "hello 🚀 world"); // 🚀 is 4 bytes

        // Exact ASCII match
        assert_eq!(sm.slice(id, Span::new(0, 5)), "hello");

        // Out of bounds clamp
        assert_eq!(sm.slice(id, Span::new(14, 100)), "ld");

        // Split multibyte char
        assert_eq!(sm.slice(id, Span::new(6, 8)), "🚀");
        assert_eq!(sm.slice(id, Span::new(6, 9)), "🚀");
        assert_eq!(sm.slice(id, Span::new(6, 10)), "🚀");
    }

    #[test]
    fn test_line_col_boundary() {
        let mut sm = SourceMap::new();
        let id = sm.add("test", "a\nb\nc");

        assert_eq!(sm.line_col(id, 0), (1, 1)); // 'a'
        assert_eq!(sm.line_col(id, 1), (1, 2)); // '\n'
        assert_eq!(sm.line_col(id, 2), (2, 1)); // 'b'
        assert_eq!(sm.line_col(id, 3), (2, 2)); // '\n'
        assert_eq!(sm.line_col(id, 4), (3, 1)); // 'c'
        assert_eq!(sm.line_col(id, 5), (3, 2)); // EOF
        assert_eq!(sm.line_col(id, 100), (3, 2)); // Clamped
    }

    #[test]
    fn test_offset_at() {
        let mut sm = SourceMap::new();
        let id = sm.add("test", "hello\nworld\n");

        assert_eq!(sm.offset_at(id, 1, 1), Some(0)); // 'h'
        assert_eq!(sm.offset_at(id, 1, 6), Some(5)); // '\n'
        assert_eq!(sm.offset_at(id, 2, 1), Some(6)); // 'w'
        assert_eq!(sm.offset_at(id, 2, 6), Some(11)); // '\n'
        assert_eq!(sm.offset_at(id, 3, 1), Some(12)); // EOF

        assert_eq!(sm.offset_at(id, 0, 1), None); // Invalid line
        assert_eq!(sm.offset_at(id, 1, 0), None); // Invalid col
        assert_eq!(sm.offset_at(id, 4, 1), None); // Out of bounds line
        assert_eq!(sm.offset_at(id, 1, 7), None); // Out of bounds col
    }

    #[test]
    fn test_round_trip() {
        let mut sm = SourceMap::new();
        let id = sm.add("test", "a\nb\ncd\ne");

        for offset in 0..=sm.source(id).len() {
            let (line, col) = sm.line_col(id, offset);
            assert_eq!(
                sm.offset_at(id, line, col),
                Some(offset),
                "failed at offset {}",
                offset
            );
        }
    }

    #[test]
    fn test_map_span() {
        let mut sm = SourceMap::new();
        let id1 = sm.add("f1", "a\nbc\nd");
        let id2 = sm.add("f2", "x\nyyy\nz"); // Longer second line

        // Same line/col span
        let span = Span::new(2, 4); // "bc"
        assert_eq!(sm.line_col(id1, 2), (2, 1));
        assert_eq!(sm.line_col(id1, 4), (2, 3));

        let mapped = sm.map_span(id1, id2, span).unwrap();
        assert_eq!(sm.line_col(id2, mapped.start), (2, 1));
        assert_eq!(sm.line_col(id2, mapped.end), (2, 3));

        // Unmappable span (target too short)
        let span2 = Span::new(2, 5); // "yyy" - line 2 col 4
        let id3 = sm.add("f3", "x\ny\nz");
        assert_eq!(sm.map_span(id2, id3, span2), None);
    }
}
