//! The LSP wire form of a token stream: `SemanticTokens.data`, the flat
//! `[deltaLine, deltaStart, length, tokenType, tokenModifiers]` array over
//! the legend, in the position encoding the client negotiated.
//!
//! Pure: text and tokens in, integers out.  No LSP library is involved,
//! so any adapter — the language server, a future editor plug-in host, a
//! test — can produce a standard token result from the same stream
//! without reclassifying anything.  Byte offsets are converted here and
//! nowhere else in this crate; a multi-line token (a block comment) is
//! split per line because the encoding is per line.

use super::SemanticToken;
use bdl_ide_db::TextRange;

/// How a client counts characters within a line (LSP 3.17
/// `general.positionEncodings`).  The default is UTF-16.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PositionEncoding {
    Utf8,
    #[default]
    Utf16,
    Utf32,
}

impl PositionEncoding {
    fn units(self, c: char) -> u32 {
        match self {
            PositionEncoding::Utf8 => c.len_utf8() as u32,
            PositionEncoding::Utf16 => c.len_utf16() as u32,
            PositionEncoding::Utf32 => 1,
        }
    }
}

/// A line/column position in the negotiated encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

/// Line starts of one text, for byte → position conversion.
pub struct LineIndex<'a> {
    text: &'a str,
    line_starts: Vec<u32>,
    encoding: PositionEncoding,
}

impl<'a> LineIndex<'a> {
    pub fn new(text: &'a str, encoding: PositionEncoding) -> LineIndex<'a> {
        let mut line_starts = vec![0u32];
        for (i, b) in text.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(i as u32 + 1);
            }
        }
        LineIndex {
            text,
            line_starts,
            encoding,
        }
    }

    /// The position of a byte offset (clamped to the text; an offset
    /// inside a character rounds down to the character's start).
    pub fn position(&self, offset: u32) -> Position {
        let offset = (offset as usize).min(self.text.len());
        let line = match self.line_starts.binary_search(&(offset as u32)) {
            Ok(i) => i,
            Err(i) => i.saturating_sub(1),
        };
        let start = self.line_starts[line] as usize;
        let boundary = (start..=offset)
            .rev()
            .find(|i| self.text.is_char_boundary(*i))
            .unwrap_or(start);
        let character = self.text[start..boundary]
            .chars()
            .map(|c| self.encoding.units(c))
            .sum();
        Position {
            line: line as u32,
            character,
        }
    }
}

/// One encoded token, before flattening.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EncodedToken {
    pub delta_line: u32,
    pub delta_start: u32,
    pub length: u32,
    pub token_type: u32,
    pub token_modifiers: u32,
}

/// Encode a sorted, disjoint token stream over `text`.
pub fn encode(
    tokens: &[SemanticToken],
    text: &str,
    encoding: PositionEncoding,
) -> Vec<EncodedToken> {
    let index = LineIndex::new(text, encoding);
    let len = text.len() as u32;
    let mut out = Vec::with_capacity(tokens.len());
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;
    for t in tokens {
        let range = t.range.clamp_to(len);
        let mut piece_start = range.start;
        loop {
            let s_pos = index.position(piece_start);
            let line_end = text[piece_start as usize..range.end as usize]
                .find('\n')
                .map(|i| piece_start + i as u32)
                .unwrap_or(range.end);
            let e_pos = index.position(line_end);
            let length = e_pos.character.saturating_sub(s_pos.character);
            if length > 0 {
                let delta_line = s_pos.line - prev_line;
                let delta_start = if delta_line == 0 {
                    s_pos.character - prev_start
                } else {
                    s_pos.character
                };
                out.push(EncodedToken {
                    delta_line,
                    delta_start,
                    length,
                    token_type: t.ty.index(),
                    token_modifiers: t.modifiers.bits(),
                });
                prev_line = s_pos.line;
                prev_start = s_pos.character;
            }
            if line_end >= range.end {
                break;
            }
            piece_start = line_end + 1;
        }
    }
    out
}

/// The flat `data` array.
pub fn encode_data(tokens: &[SemanticToken], text: &str, encoding: PositionEncoding) -> Vec<u32> {
    encode(tokens, text, encoding)
        .into_iter()
        .flat_map(|t| {
            [
                t.delta_line,
                t.delta_start,
                t.length,
                t.token_type,
                t.token_modifiers,
            ]
        })
        .collect()
}

/// Decode a `data` array back into absolute (line, character, length,
/// type, modifiers) rows — for tests and for clients that want rows.
pub fn decode_data(data: &[u32]) -> Vec<(u32, u32, u32, u32, u32)> {
    let mut out = Vec::new();
    let mut line = 0u32;
    let mut start = 0u32;
    for row in data.chunks_exact(5) {
        line += row[0];
        start = if row[0] == 0 { start + row[1] } else { row[1] };
        out.push((line, start, row[2], row[3], row[4]));
    }
    out
}

/// A token's byte range as `(start, end)` for a decoded row, given the
/// same text and encoding — the inverse of [`encode`], for round-trips.
pub fn byte_range(
    text: &str,
    encoding: PositionEncoding,
    line: u32,
    character: u32,
    length: u32,
) -> Option<TextRange> {
    let mut lines = text.split_inclusive('\n');
    let mut offset = 0u32;
    for _ in 0..line {
        offset += lines.next()?.len() as u32;
    }
    let line_text = lines.next().unwrap_or("");
    let mut units = 0u32;
    let mut start = None;
    let mut end = None;
    let mut bytes = 0u32;
    for c in line_text.chars().chain(std::iter::once('\0')) {
        if start.is_none() && units >= character {
            start = Some(bytes);
        }
        if start.is_some() && units >= character + length {
            end = Some(bytes);
            break;
        }
        if c == '\0' {
            break;
        }
        units += encoding.units(c);
        bytes += c.len_utf8() as u32;
    }
    Some(TextRange::new(offset + start?, offset + end?))
}

#[cfg(test)]
mod tests {
    use super::super::{SemanticToken, TokenModifiers, TokenType};
    use super::*;

    fn tok(start: u32, end: u32, ty: TokenType) -> SemanticToken {
        SemanticToken {
            range: TextRange::new(start, end),
            ty,
            modifiers: TokenModifiers::default(),
        }
    }

    const SRC: &str =
        "// ångström ✓ 𝛼\nconcept Tilt : Angle\n// 日本語\nmapping f : Tilt -> Tilt\n";

    fn find(s: &str, needle: &str) -> u32 {
        s.find(needle).expect("needle") as u32
    }

    #[test]
    fn every_encoding_round_trips_byte_ranges_on_non_ascii_text() {
        let tokens = vec![
            tok(0, find(SRC, "\n"), TokenType::Comment),
            tok(
                find(SRC, "concept"),
                find(SRC, "concept") + 7,
                TokenType::Keyword,
            ),
            tok(find(SRC, "Tilt"), find(SRC, "Tilt") + 4, TokenType::Type),
            tok(find(SRC, "Angle"), find(SRC, "Angle") + 5, TokenType::Type),
            tok(
                find(SRC, "// 日本語"),
                find(SRC, "// 日本語") + "// 日本語".len() as u32,
                TokenType::Comment,
            ),
            tok(
                find(SRC, "mapping"),
                find(SRC, "mapping") + 7,
                TokenType::Keyword,
            ),
            tok(
                find(SRC, "-> Tilt") + 3,
                find(SRC, "-> Tilt") + 7,
                TokenType::Type,
            ),
        ];
        for enc in [
            PositionEncoding::Utf8,
            PositionEncoding::Utf16,
            PositionEncoding::Utf32,
        ] {
            let data = encode_data(&tokens, SRC, enc);
            assert_eq!(data.len(), tokens.len() * 5);
            let rows = decode_data(&data);
            for (row, t) in rows.iter().zip(&tokens) {
                let back = byte_range(SRC, enc, row.0, row.1, row.2).expect("range");
                assert_eq!(back, t.range, "{enc:?} {row:?}");
                assert_eq!(row.3, t.ty.index());
            }
        }
        // The CJK comment: three chars, three UTF-16 units, nine bytes.
        let utf16 = decode_data(&encode_data(&tokens, SRC, PositionEncoding::Utf16));
        assert_eq!(utf16[4], (2, 0, 6, TokenType::Comment.index(), 0));
        let utf8 = decode_data(&encode_data(&tokens, SRC, PositionEncoding::Utf8));
        assert_eq!(utf8[4].2, 12);
        // The astral alpha on line 0 counts two UTF-16 units, one UTF-32 unit.
        assert_eq!(utf16[0].2, "// ångström ✓ 𝛼".encode_utf16().count() as u32);
        let utf32 = decode_data(&encode_data(&tokens, SRC, PositionEncoding::Utf32));
        assert_eq!(utf32[0].2, "// ångström ✓ 𝛼".chars().count() as u32);
    }

    #[test]
    fn multi_line_tokens_split_per_line_and_deltas_are_relative() {
        let text = "/* a\n  b */ x";
        let tokens = vec![tok(0, 11, TokenType::Comment), tok(12, 13, TokenType::Type)];
        let rows = decode_data(&encode_data(&tokens, text, PositionEncoding::Utf16));
        assert_eq!(
            rows,
            vec![
                (0, 0, 4, TokenType::Comment.index(), 0),
                (1, 0, 6, TokenType::Comment.index(), 0),
                (1, 7, 1, TokenType::Type.index(), 0),
            ]
        );
        let raw = encode(&tokens, text, PositionEncoding::Utf16);
        assert_eq!(raw[2].delta_line, 0);
        assert_eq!(raw[2].delta_start, 7);
    }

    #[test]
    fn stale_or_out_of_range_tokens_are_clamped_never_panicking() {
        let text = "ab";
        let tokens = vec![
            tok(1, 99, TokenType::Number),
            tok(50, 60, TokenType::Number),
        ];
        let rows = decode_data(&encode_data(&tokens, text, PositionEncoding::Utf16));
        assert_eq!(rows, vec![(0, 1, 1, TokenType::Number.index(), 0)]);
        assert!(encode_data(&[], "", PositionEncoding::Utf8).is_empty());
        let inside = LineIndex::new("é", PositionEncoding::Utf16).position(1);
        assert_eq!(
            inside,
            Position {
                line: 0,
                character: 0
            }
        );
    }
}
