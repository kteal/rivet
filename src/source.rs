use std::path::{Path, PathBuf};

pub const DUMMY_FILE_ID: FileId = FileId(0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub file_id: FileId,
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[must_use]
    pub const fn new(file_id: FileId, start: usize, end: usize) -> Self {
        Self {
            file_id,
            start,
            end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    line_starts: Vec<usize>,
}

impl SourceFile {
    #[must_use]
    pub fn new(path: &Path, text: String) -> Self {
        let line_starts = Self::compute_line_starts(&text);

        Self {
            path: path.to_path_buf(),
            text,
            line_starts,
        }
    }

    fn compute_line_starts(text: &str) -> Vec<usize> {
        let mut line_starts = vec![0];

        for (byte_index, ch) in text.char_indices() {
            if ch == '\n' {
                line_starts.push(byte_index + 1);
            }
        }

        line_starts
    }

    #[must_use]
    pub fn line_col(&self, offset: usize) -> (usize, usize) {
        let mut line_index = 0;
        for (start_index, line_start) in self.line_starts.iter().enumerate() {
            if *line_start <= offset {
                line_index = start_index;
            } else {
                break;
            }
        }
        let line_start = self.line_starts[line_index];
        (line_index + 1, offset - line_start + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceLocation {
    pub path: PathBuf,
    pub line: usize,
    pub column: usize,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    #[must_use]
    pub const fn new() -> Self {
        Self { files: vec![] }
    }

    pub fn add_file(&mut self, path: &Path, text: String) -> FileId {
        let file = SourceFile::new(path, text);
        let file_id = FileId(self.files.len());
        self.files.push(file);
        file_id
    }

    /// Returns the source file for `file_id`.
    ///
    /// # Panics
    ///
    /// Panics if `file_id` does not refer to a file in this source map.
    #[must_use]
    pub fn file(&self, file_id: FileId) -> &SourceFile {
        self.files.get(file_id.0).expect("file does not exist")
    }

    #[must_use]
    pub fn location(&self, span: Span) -> SourceLocation {
        let file = self.file(span.file_id);
        let (line, col) = file.line_col(span.start);

        SourceLocation {
            path: file.path.clone(),
            line,
            column: col,
        }
    }
}
