//! Source span and position tracking for precise error reporting and LSP features

use serde::{Deserialize, Serialize};
use std::fmt;

/// Unique identifier for a source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FileId(pub u32);

impl FileId {
    pub const INVALID: FileId = FileId(u32::MAX);
    
    pub fn new(id: u32) -> Self {
        FileId(id)
    }
    
    pub fn as_u32(self) -> u32 {
        self.0
    }
}

impl fmt::Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file:{}", self.0)
    }
}

/// Byte offset in a source file (0-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ByteOffset(pub u32);

impl ByteOffset {
    pub const INVALID: ByteOffset = ByteOffset(u32::MAX);
    
    pub fn new(offset: u32) -> Self {
        ByteOffset(offset)
    }
    
    pub fn as_u32(self) -> u32 {
        self.0
    }
    
    pub fn advance(self, by: u32) -> Self {
        ByteOffset(self.0 + by)
    }
}

/// Line number in a source file (0-based for internal use, 1-based for display)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Line(pub u32);

impl Line {
    pub fn new(line: u32) -> Self {
        Line(line)
    }
    
    pub fn as_u32(self) -> u32 {
        self.0
    }
    
    /// Convert to 1-based line number for display
    pub fn to_display(self) -> u32 {
        self.0 + 1
    }
}

/// Column number in a source file (0-based for internal use, 1-based for display)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Column(pub u32);

impl Column {
    pub fn new(column: u32) -> Self {
        Column(column)
    }
    
    pub fn as_u32(self) -> u32 {
        self.0
    }
    
    /// Convert to 1-based column number for display
    pub fn to_display(self) -> u32 {
        self.0 + 1
    }
}

/// Line and column position in a source file
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Position {
    pub line: Line,
    pub column: Column,
}

impl Position {
    pub fn new(line: u32, column: u32) -> Self {
        Position {
            line: Line(line),
            column: Column(column),
        }
    }
    
    pub fn zero() -> Self {
        Position::new(0, 0)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line.to_display(), self.column.to_display())
    }
}

/// Conversion to LSP Position
impl From<Position> for lsp_types::Position {
    fn from(pos: Position) -> Self {
        lsp_types::Position {
            line: pos.line.as_u32(),
            character: pos.column.as_u32(),
        }
    }
}

impl From<lsp_types::Position> for Position {
    fn from(pos: lsp_types::Position) -> Self {
        Position {
            line: Line(pos.line),
            column: Column(pos.character),
        }
    }
}

/// Span in a source file (inclusive start, exclusive end)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub file_id: FileId,
    pub start: ByteOffset,
    pub end: ByteOffset,
}

impl Span {
    pub fn new(file_id: FileId, start: ByteOffset, end: ByteOffset) -> Self {
        Span { file_id, start, end }
    }
    
    pub fn single(file_id: FileId, offset: ByteOffset) -> Self {
        Span {
            file_id,
            start: offset,
            end: offset.advance(1),
        }
    }
    
    pub fn len(self) -> u32 {
        self.end.0.saturating_sub(self.start.0)
    }
    
    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
    
    /// Combine two spans into a span that covers both
    pub fn merge(self, other: Span) -> Span {
        assert_eq!(self.file_id, other.file_id);
        Span {
            file_id: self.file_id,
            start: ByteOffset(self.start.0.min(other.start.0)),
            end: ByteOffset(self.end.0.max(other.end.0)),
        }
    }
    
    /// Check if this span contains the given offset
    pub fn contains(self, offset: ByteOffset) -> bool {
        self.start <= offset && offset < self.end
    }
    
    /// Check if this span contains the given position
    pub fn contains_position(self, other: Span) -> bool {
        self.file_id == other.file_id && 
        self.start <= other.start && 
        other.end <= self.end
    }
    
    /// Check if this span overlaps with another span
    pub fn overlaps(self, other: Span) -> bool {
        self.file_id == other.file_id &&
        self.start < other.end &&
        other.start < self.end
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}..{}", self.file_id, self.start.0, self.end.0)
    }
}

/// Conversion to LSP Range (requires position mapping)
impl Span {
    pub fn to_lsp_range(self, line_map: &LineMap) -> lsp_types::Range {
        let start_pos = line_map.offset_to_position(self.start);
        let end_pos = line_map.offset_to_position(self.end);
        
        lsp_types::Range {
            start: start_pos.into(),
            end: end_pos.into(),
        }
    }
}

/// Mapping between byte offsets and line/column positions
#[derive(Debug, Clone)]
pub struct LineMap {
    /// Byte offsets of the start of each line
    line_starts: Vec<ByteOffset>,
}

impl LineMap {
    pub fn new(source: &str) -> Self {
        let mut line_starts = vec![ByteOffset(0)];
        
        for (idx, byte) in source.bytes().enumerate() {
            if byte == b'\n' {
                line_starts.push(ByteOffset(idx as u32 + 1));
            }
        }
        
        LineMap { line_starts }
    }
    
    pub fn line_count(&self) -> u32 {
        self.line_starts.len() as u32
    }
    
    pub fn offset_to_position(&self, offset: ByteOffset) -> Position {
        // Binary search for the line containing the offset
        match self.line_starts.binary_search(&offset) {
            Ok(line) => Position {
                line: Line(line as u32),
                column: Column(0),
            },
            Err(line) => {
                let line = line.saturating_sub(1);
                let line_start = self.line_starts[line];
                Position {
                    line: Line(line as u32),
                    column: Column(offset.0.saturating_sub(line_start.0)),
                }
            }
        }
    }
    
    pub fn position_to_offset(&self, position: Position) -> Option<ByteOffset> {
        let line_idx = position.line.as_u32() as usize;
        
        if line_idx >= self.line_starts.len() {
            return None;
        }
        
        let line_start = self.line_starts[line_idx];
        Some(line_start.advance(position.column.as_u32()))
    }
    
    pub fn line_span(&self, line: Line) -> Option<Span> {
        let line_idx = line.as_u32() as usize;
        
        if line_idx >= self.line_starts.len() {
            return None;
        }
        
        let start = self.line_starts[line_idx];
        let end = self.line_starts.get(line_idx + 1)
            .copied()
            .unwrap_or_else(|| ByteOffset(u32::MAX));
        
        Some(Span::new(FileId::INVALID, start, end))
    }
}

/// Trait for AST nodes that have source spans
pub trait HasSpan {
    fn span(&self) -> Span;
}

/// Wrapper type that adds span information to any value
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub value: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(value: T, span: Span) -> Self {
        Spanned { value, span }
    }
    
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> Spanned<U> {
        Spanned {
            value: f(self.value),
            span: self.span,
        }
    }
    
    pub fn as_ref(&self) -> Spanned<&T> {
        Spanned {
            value: &self.value,
            span: self.span,
        }
    }
}

impl<T> HasSpan for Spanned<T> {
    fn span(&self) -> Span {
        self.span
    }
}

impl<T: fmt::Display> fmt::Display for Spanned<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.value, self.span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_map() {
        let source = "hello\nworld\n\nfoo";
        let line_map = LineMap::new(source);
        
        assert_eq!(line_map.line_count(), 4);
        
        // Test offset to position conversion
        assert_eq!(
            line_map.offset_to_position(ByteOffset(0)),
            Position::new(0, 0)
        );
        assert_eq!(
            line_map.offset_to_position(ByteOffset(6)),
            Position::new(1, 0)
        );
        assert_eq!(
            line_map.offset_to_position(ByteOffset(12)),
            Position::new(2, 0)
        );
        
        // Test position to offset conversion
        assert_eq!(
            line_map.position_to_offset(Position::new(0, 0)),
            Some(ByteOffset(0))
        );
        assert_eq!(
            line_map.position_to_offset(Position::new(1, 0)),
            Some(ByteOffset(6))
        );
    }

    #[test]
    fn test_span_operations() {
        let file_id = FileId::new(1);
        let span1 = Span::new(file_id, ByteOffset(0), ByteOffset(5));
        let span2 = Span::new(file_id, ByteOffset(3), ByteOffset(8));
        
        // Test merge
        let merged = span1.merge(span2);
        assert_eq!(merged.start, ByteOffset(0));
        assert_eq!(merged.end, ByteOffset(8));
        
        // Test contains
        assert!(span1.contains(ByteOffset(2)));
        assert!(!span1.contains(ByteOffset(5)));
        
        // Test overlaps
        assert!(span1.overlaps(span2));
        
        let span3 = Span::new(file_id, ByteOffset(10), ByteOffset(15));
        assert!(!span1.overlaps(span3));
    }
}