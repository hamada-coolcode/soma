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

    /// 1-based (line, column) for a byte offset.
    pub fn line_col(&self, id: FileId, offset: usize) -> (usize, usize) {
        let text = self.source(id);
        let upto = offset.min(text.len());
        let line = text[..upto].bytes().filter(|b| *b == b'\n').count() + 1;
        let col = upto - text[..upto].rfind('\n').map(|i| i + 1).unwrap_or(0) + 1;
        (line, col)
    }
}
