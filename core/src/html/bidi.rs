//! Unicode bidirectional ordering for text layout.
//!
//! The output maps each visual character back to its original UTF-16
//! code-unit offset. It does not modify the logical source text.

use unicode_bidi::{BidiInfo, Level};

/// Returns logical UTF-16 offsets in visual left-to-right order.
///
/// This only resolves bidirectional ordering. Glyph shaping, mirroring,
/// and layout positioning are separate responsibilities.
pub(super) fn visual_order(text: &str, base_level: u8) -> Vec<usize> {
    let base_level = Level::new(base_level).unwrap_or(Level::ltr());

    let bidi = BidiInfo::new(text, Some(base_level));

    // Map UTF-8 character byte offsets to UTF-16 logical offsets.
    let mut byte_to_utf16 = Vec::new();
    let mut utf16_offset = 0;

    for (byte_offset, ch) in text.char_indices() {
        byte_to_utf16.push((byte_offset, utf16_offset));
        utf16_offset += ch.len_utf16();
    }

    let mut result = Vec::new();

    for paragraph in &bidi.paragraphs {
        let (levels, runs) = bidi.visual_runs(paragraph, paragraph.range.clone());

        for run in runs {
            let mut offsets: Vec<usize> = byte_to_utf16
                .iter()
                .filter(|(byte, _)| run.contains(byte))
                .map(|(_, logical_offset)| *logical_offset)
                .collect();

            if levels[run.start].is_rtl() {
                offsets.reverse();
            }

            result.extend(offsets);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::visual_order;

    #[test]
    fn latin_text_retains_order() {
        assert_eq!(visual_order("ABC", 0), vec![0, 1, 2]);
    }

    #[test]
    fn hebrew_run_is_reversed_visually() {
        assert_eq!(visual_order("שלום", 1), vec![3, 2, 1, 0]);
    }

    #[test]
    fn supplementary_characters_keep_utf16_offsets() {
        assert_eq!(visual_order("A😀B", 0), vec![0, 1, 3]);
    }

    #[test]
    fn bidi_level_changes_mixed_text_order() {
        let text = "ABC שלום 123";

        let ltr = visual_order(text, 0);
        let rtl = visual_order(text, 1);

        assert_ne!(ltr, rtl);

        let mut sorted_ltr = ltr;
        let mut sorted_rtl = rtl;

        sorted_ltr.sort_unstable();
        sorted_rtl.sort_unstable();

        assert_eq!(sorted_ltr, sorted_rtl);
    }
    #[test]
    fn hebrew_latin_digits_have_different_visual_orders() {
        let text = "ABC שלום 123";

        let ltr = visual_order(text, 0);
        let rtl = visual_order(text, 1);

        let chars: Vec<char> = text.chars().collect();

        let ltr_visual: String = ltr.iter().map(|&i| chars[i]).collect();
        let rtl_visual: String = rtl.iter().map(|&i| chars[i]).collect();

        assert_eq!(ltr_visual, "ABC 123 םולש");
        assert_eq!(rtl_visual, "123 םולש ABC");
    }

    #[test]
    fn logical_indices_are_preserved_after_reordering() {
        let text = "ABC שלום 123";

        for level in [0, 1] {
            let mut positions = visual_order(text, level);

            positions.sort_unstable();

            assert_eq!(
                positions,
                (0..text.encode_utf16().count()).collect::<Vec<_>>()
            );
        }
    }
}
