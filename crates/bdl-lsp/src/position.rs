//! The one place byte offsets meet LSP positions.
//!
//! The IDE core speaks UTF-8 byte offsets.  LSP speaks `(line, character)`
//! where `character` counts code units of the *negotiated* encoding —
//! UTF-16 by default, UTF-8 or UTF-32 when both sides agree (LSP 3.17
//! `general.positionEncodings`).  A [`LineIndex`] converts both ways for
//! one document; nothing else in the adapter touches the arithmetic.
//!
//! Clamping, never panicking: a position past the end of a line clamps to
//! the line end, past the last line to the document end; an offset inside
//! a multi-byte character rounds down to its start.

use bdl_ide_db::TextRange;
use lsp_types::{Position, PositionEncodingKind, Range};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PositionEncoding {
    Utf8,
    Utf16,
    Utf32,
}

impl PositionEncoding {
    /// Pick from what the client offers: UTF-8 when it is offered (no
    /// conversion at all), else UTF-16 (the protocol default), else
    /// UTF-32.
    pub fn negotiate(offered: Option<&[PositionEncodingKind]>) -> PositionEncoding {
        let Some(offered) = offered else {
            return PositionEncoding::Utf16;
        };
        if offered.contains(&PositionEncodingKind::UTF8) {
            PositionEncoding::Utf8
        } else if offered.contains(&PositionEncodingKind::UTF16) || offered.is_empty() {
            PositionEncoding::Utf16
        } else if offered.contains(&PositionEncodingKind::UTF32) {
            PositionEncoding::Utf32
        } else {
            PositionEncoding::Utf16
        }
    }

    pub fn kind(self) -> PositionEncodingKind {
        match self {
            PositionEncoding::Utf8 => PositionEncodingKind::UTF8,
            PositionEncoding::Utf16 => PositionEncodingKind::UTF16,
            PositionEncoding::Utf32 => PositionEncodingKind::UTF32,
        }
    }

    /// Code units one character occupies in this encoding.
    fn units(self, c: char) -> u32 {
        match self {
            PositionEncoding::Utf8 => c.len_utf8() as u32,
            PositionEncoding::Utf16 => c.len_utf16() as u32,
            PositionEncoding::Utf32 => 1,
        }
    }
}

/// Line starts of one document, for both conversions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineIndex {
    /// Byte offset of the start of each line; `[0]` is always `0`.
    line_starts: Vec<u32>,
    len: u32,
    encoding: PositionEncoding,
    text: String,
}

impl LineIndex {
    pub fn new(text: &str, encoding: PositionEncoding) -> LineIndex {
        let mut line_starts = vec![0u32];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        LineIndex {
            line_starts,
            len: text.len() as u32,
            encoding,
            text: text.to_owned(),
        }
    }

    pub fn encoding(&self) -> PositionEncoding {
        self.encoding
    }

    fn line_of(&self, offset: u32) -> usize {
        match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        }
    }

    fn line_text(&self, line: usize) -> &str {
        let start = self.line_starts[line] as usize;
        let end = self
            .line_starts
            .get(line + 1)
            .map(|e| *e as usize)
            .unwrap_or(self.text.len());
        &self.text[start..end]
    }

    /// Byte offset → position in the negotiated encoding.
    pub fn position(&self, offset: u32) -> Position {
        let offset = offset.min(self.len);
        let line = self.line_of(offset);
        let line_start = self.line_starts[line];
        let prefix_end = (offset - line_start) as usize;
        let text = self.line_text(line);
        // Round down to a character boundary.
        let prefix_end = (0..=prefix_end.min(text.len()))
            .rev()
            .find(|i| text.is_char_boundary(*i))
            .unwrap_or(0);
        let character = text[..prefix_end]
            .chars()
            .map(|c| self.encoding.units(c))
            .sum();
        Position {
            line: line as u32,
            character,
        }
    }

    /// Position in the negotiated encoding → byte offset, clamped.
    pub fn offset(&self, position: Position) -> u32 {
        let line = (position.line as usize).min(self.line_starts.len() - 1);
        let text = self.line_text(line);
        let mut units = 0u32;
        let mut bytes = 0usize;
        for c in text.chars() {
            if c == '\n' || units >= position.character {
                break;
            }
            let next = units + self.encoding.units(c);
            if next > position.character {
                // Inside a character (a lone UTF-16 surrogate index): its start.
                break;
            }
            units = next;
            bytes += c.len_utf8();
        }
        self.line_starts[line] + bytes as u32
    }

    pub fn range(&self, r: TextRange) -> Range {
        Range {
            start: self.position(r.start),
            end: self.position(r.end),
        }
    }

    pub fn text_range(&self, r: Range) -> TextRange {
        let start = self.offset(r.start);
        let end = self.offset(r.end).max(start);
        TextRange::new(start, end)
    }

    pub fn len(&self) -> u32 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRC: &str =
        "// ångström ✓ 𝛼\nconcept Tilt : Angle\n// 日本語\nmapping f : Tilt -> Tilt\n";

    fn byte_of(s: &str, needle: &str) -> u32 {
        s.find(needle).expect("needle present") as u32
    }

    #[test]
    fn round_trips_every_boundary_in_every_encoding() {
        for enc in [
            PositionEncoding::Utf8,
            PositionEncoding::Utf16,
            PositionEncoding::Utf32,
        ] {
            let ix = LineIndex::new(SRC, enc);
            for (i, _) in SRC.char_indices() {
                let p = ix.position(i as u32);
                assert_eq!(ix.offset(p), i as u32, "{enc:?} offset {i}");
            }
            let end = ix.position(SRC.len() as u32);
            assert_eq!(ix.offset(end), SRC.len() as u32);
        }
    }

    #[test]
    fn utf16_counts_surrogate_pairs_and_utf8_counts_bytes() {
        let tilt = byte_of(SRC, "Tilt");
        let ix16 = LineIndex::new(SRC, PositionEncoding::Utf16);
        let ix8 = LineIndex::new(SRC, PositionEncoding::Utf8);
        let ix32 = LineIndex::new(SRC, PositionEncoding::Utf32);
        assert_eq!(ix16.position(tilt), Position::new(1, 8));
        assert_eq!(ix8.position(tilt), Position::new(1, 8));
        // The mathematical alpha on line 0 is one char, two UTF-16 units,
        // four bytes.
        let alpha = byte_of(SRC, "𝛼");
        assert_eq!(
            ix32.position(alpha).character + 1,
            ix32.position(alpha + 4).character
        );
        assert_eq!(
            ix16.position(alpha).character + 2,
            ix16.position(alpha + 4).character
        );
        assert_eq!(
            ix8.position(alpha).character + 4,
            ix8.position(alpha + 4).character
        );
        // Line 2 is CJK: three chars, three UTF-16 units, nine bytes.
        let f = byte_of(SRC, "mapping f");
        assert_eq!(ix16.position(f), Position::new(3, 0));
        let ja_end = byte_of(SRC, "日本語") + 9;
        assert_eq!(ix16.position(ja_end), Position::new(2, 6));
        assert_eq!(ix8.position(ja_end), Position::new(2, 12));
        assert_eq!(ix32.position(ja_end), Position::new(2, 6));
    }

    #[test]
    fn out_of_range_positions_clamp_instead_of_panicking() {
        let ix = LineIndex::new(SRC, PositionEncoding::Utf16);
        assert_eq!(ix.offset(Position::new(99, 99)), SRC.len() as u32);
        let line1_end = byte_of(SRC, "Angle") + 5;
        assert_eq!(ix.offset(Position::new(1, 500)), line1_end);
        // Inside the surrogate pair: the character's start.
        let alpha = byte_of(SRC, "𝛼");
        let p = ix.position(alpha);
        assert_eq!(ix.offset(Position::new(p.line, p.character + 1)), alpha);
        // An offset inside a multibyte char rounds down.
        let a_ring = byte_of(SRC, "å");
        assert_eq!(ix.position(a_ring + 1), ix.position(a_ring));
        assert_eq!(ix.position(u32::MAX), ix.position(SRC.len() as u32));
        let empty = LineIndex::new("", PositionEncoding::Utf16);
        assert_eq!(empty.offset(Position::new(3, 3)), 0);
        assert_eq!(empty.position(7), Position::new(0, 0));
    }

    #[test]
    fn negotiation_prefers_utf8_then_utf16() {
        assert_eq!(PositionEncoding::negotiate(None), PositionEncoding::Utf16);
        assert_eq!(
            PositionEncoding::negotiate(Some(&[
                PositionEncodingKind::UTF16,
                PositionEncodingKind::UTF8
            ])),
            PositionEncoding::Utf8
        );
        assert_eq!(
            PositionEncoding::negotiate(Some(&[PositionEncodingKind::UTF32])),
            PositionEncoding::Utf32
        );
    }
}
