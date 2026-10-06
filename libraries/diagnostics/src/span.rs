use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub const fn empty() -> Self {
        Self { start: 0, end: 0 }
    }

    pub const fn len(&self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    pub const fn is_empty(&self) -> bool {
        self.end <= self.start
    }

    pub fn contains(&self, offset: u32) -> bool {
        offset >= self.start && offset < self.end.max(self.start + 1)
    }

    pub const fn overlaps(&self, other: Span) -> bool {
        self.start < other.end && other.start < self.end
    }

    pub fn join(self, other: Span) -> Span {
        Span::new(self.start.min(other.start), self.end.max(other.end))
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

#[derive(Clone, PartialEq)]
pub struct SourceFile {
    pub path: String,
    pub name: String,
    pub text: String,
    line_starts: Vec<u32>,
}

impl SourceFile {
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        let path = path.into();
        let name = path
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(path.as_str())
            .to_string();
        let text = text.into();
        let line_starts = compute_line_starts(&text);
        Self {
            path,
            name,
            text,
            line_starts,
        }
    }

    pub fn line_col(&self, offset: u32) -> (u32, u32) {
        let offset = offset.min(self.text.len() as u32);
        let line_index = match self.line_starts.binary_search(&offset) {
            Ok(index) => index,
            Err(index) => index.saturating_sub(1),
        };
        let line_start = self.line_starts[line_index];
        (line_index as u32 + 1, offset - line_start + 1)
    }

    pub fn line_of(&self, offset: u32) -> u32 {
        self.line_col(offset).0
    }

    pub fn line_text(&self, line: u32) -> &str {
        if line == 0 || line as usize > self.line_starts.len() {
            return "";
        }
        let start = self.line_starts[line as usize - 1] as usize;
        let end = self
            .line_starts
            .get(line as usize)
            .map(|next| *next as usize)
            .unwrap_or(self.text.len());
        self.text[start..end].trim_end_matches(['\n', '\r'])
    }

    pub fn describe(&self, span: Span) -> String {
        let (line, column) = self.line_col(span.start);
        format!("{}:{}:{}", self.name, line, column)
    }

    pub fn caret_line(&self, span: Span) -> String {
        let line = self.line_of(span.start);
        let text = self.line_text(line);
        let indent = text.len() - text.trim_start().len();
        let width = span.len().max(1) as usize;
        let mut marker = " ".repeat(indent + (self.line_col(span.start).1 - 1) as usize);
        marker.push('^');
        if width > 1 {
            marker.push_str(&"~".repeat(width - 1));
        }
        format!("{text}\n{marker}")
    }
}

impl fmt::Debug for SourceFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SourceFile")
            .field("path", &self.path)
            .field("lines", &self.line_starts.len())
            .finish()
    }
}

fn compute_line_starts(text: &str) -> Vec<u32> {
    let mut starts = vec![0];
    for (offset, byte) in text.bytes().enumerate() {
        if byte == b'\n' {
            starts.push(offset as u32 + 1);
        }
    }
    starts
}
