//! Places in a text as Monaco counts them, 1-based lines and UTF-16 columns, against byte offsets.

/// The 1-based line and UTF-16 column just after byte `offset` of `text`.
pub(crate) fn position(text: &str, offset: usize) -> (u32, u32) {
    let before = &text[..offset];
    let line_start = before.rfind('\n').map_or(0, |n| n + 1);
    let line = before.matches('\n').count() as u32 + 1;
    let column = before[line_start..].encode_utf16().count() as u32 + 1;
    (line, column)
}

/// The byte offset of 1-based `line` and UTF-16 `column` in `text`; `None` past a line's end or the text's.
pub(crate) fn offset(text: &str, line: u32, column: u32) -> Option<usize> {
    let start = if line == 1 {
        0
    } else {
        text.match_indices('\n')
            .nth(line.checked_sub(2)? as usize)?
            .0
            + 1
    };
    let rest = &text[start..];
    let line_text = &rest[..rest.find('\n').unwrap_or(rest.len())];
    let mut units = 0;
    for (at, c) in line_text.char_indices() {
        if units + 1 == column {
            return Some(start + at);
        }
        units += c.len_utf16() as u32;
    }
    (units + 1 == column).then_some(start + line_text.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_offset_and_its_position_meet_in_both_directions() {
        let text = "ab\nµx€y\n\nz";
        for at in [0, 1, 2, 3, 5, 6, 9, 10, 12, 13] {
            let (line, column) = position(text, at);
            assert_eq!(
                offset(text, line, column),
                Some(at),
                "{at} at {line}:{column}"
            );
        }
        assert_eq!(position(text, 6), (2, 3), "µ is one UTF-16 unit");
        assert_eq!(offset(text, 2, 5), Some(10), "the line's end");
        assert_eq!(offset(text, 2, 6), None, "past the line's end");
        assert_eq!(offset(text, 5, 1), None, "past the text's end");
        assert_eq!(offset(text, 0, 1), None);
        assert_eq!(offset("😀a", 1, 3), Some(4), "an emoji is two UTF-16 units");
    }
}
