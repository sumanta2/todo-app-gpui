//! Per-font character widths used by hit testing and cursor placement.

use crate::constants::typography::FontType;

/// Adjusts a generic weight multiplier (such as `WEIGHT_BOLD`) to how much wider heavier
/// glyphs really are in the given font.
///
/// Calibri Bold is only about 2% wider than Calibri Regular, so applying the generic 10% would
/// make the cursor drift further right with every bold character.
pub(crate) fn effective_weight_multiplier(weight_multiplier: f32, font_type: FontType) -> f32 {
    match font_type {
        FontType::Calibri => 1.0 + (weight_multiplier - 1.0) * 0.2,
        _ => weight_multiplier,
    }
}

/// Estimates the rendered width of a character for a specific font family type and size.
///
/// This provides accurate font-specific metrics across various typefaces (Monospace,
/// Neo-grotesque sans, Serifs, Segoe UI) so that cursor positioning and hit-testing align
/// precisely with GPUI's layout renderer.
pub(crate) fn get_char_width_for_font(ch: char, font_size: f32, font_type: FontType) -> f32 {
    match font_type {
        FontType::Monospace => {
            // In monospace fonts, all glyphs share an identical fixed pitch width.
            0.601 * font_size
        }
        FontType::Arial => {
            let base = match ch {
                'i' | 'l' | '\'' | '|' | '!' | '.' | ',' | ';' | ':' | '`' => 3.33,
                'I' | 'j' => 3.33,
                ' ' | '\u{00A0}' => 3.33,
                't' | 'f' | 'r' | '(' | ')' | '[' | ']' | '{' | '}' | '-' | '_' | '+' | '*'
                | '=' => 4.00,
                'J' | 's' | 'z' => 6.00,
                'c' | 'v' | 'x' | '?' | '/' | '\\' => 6.00,
                'k' => 6.00,
                'e' | 'a' | 'o' => 6.67,
                'b' | 'd' | 'g' | 'h' | 'n' | 'p' | 'q' | 'u' | 'y' => 6.67,
                '0'..='9' => 6.67,
                'L' | 'S' => 6.67,
                'F' | 'E' | 'T' | 'Z' => 6.67,
                'P' | 'Y' => 7.33,
                'B' | 'K' | 'R' | 'V' | 'X' => 8.00,
                'A' | 'C' | 'D' | 'G' | 'H' | 'N' | 'O' | 'Q' | 'U' => 8.67,
                'w' => 8.80,
                '@' | '%' | '&' | '#' | '$' => 9.33,
                'm' => 10.00,
                'M' => 10.00,
                'W' => 11.33,
                _ => 6.67,
            };
            base * (font_size / 12.0)
        }
        FontType::Serif => {
            let base = match ch {
                'i' | 'l' | '\'' | '|' | '!' | '.' | ',' | ';' | ':' | '`' => 4.00,
                'I' | 'j' => 3.80,
                ' ' | '\u{00A0}' => 3.20,
                't' | 'f' | 'r' | '(' | ')' | '[' | ']' | '{' | '}' | '-' | '_' | '+' | '*'
                | '=' => 4.20,
                'J' | 's' | 'z' => 5.33,
                'c' | 'v' | 'x' | '?' | '/' | '\\' => 5.33,
                'k' => 6.00,
                'e' | 'a' => 5.80,
                'b' | 'd' | 'g' | 'h' | 'n' | 'o' | 'p' | 'q' | 'u' | 'y' => 6.00,
                '0'..='9' => 6.00,
                'L' | 'S' => 6.67,
                'F' | 'E' | 'T' | 'Z' => 7.20,
                'P' | 'Y' => 7.50,
                'B' | 'K' | 'R' | 'V' | 'X' => 8.00,
                'A' | 'C' => 8.00,
                'G' | 'D' | 'U' | 'H' | 'N' | 'O' | 'Q' => 8.67,
                'w' => 8.67,
                '@' | '%' | '&' | '#' | '$' => 9.00,
                'm' => 9.33,
                'M' => 10.67,
                'W' => 11.33,
                _ => 6.00,
            };
            base * (font_size / 12.0)
        }
        FontType::Calibri => {
            // Advance widths at 12px taken from C:\Windows\Fonts\calibri.ttf (hmtx / unitsPerEm * 12).
            let base = match ch {
                '\'' => 2.65,
                ' ' | '\u{00A0}' => 2.71,
                'i' | 'l' => 2.75,
                'j' => 2.87,
                ',' => 2.99,
                'I' => 3.02,
                '.' => 3.03,
                ';' | ':' => 3.21,
                '`' => 3.49,
                '(' | ')' => 3.64,
                'f' => 3.66,
                '-' => 3.67,
                '[' | ']' => 3.68,
                '{' | '}' => 3.77,
                'J' => 3.83,
                '!' => 3.91,
                't' => 4.02,
                'r' => 4.18,
                '/' | '\\' => 4.63,
                's' => 4.69,
                'z' => 4.74,
                'L' => 5.04,
                'c' => 5.07,
                'x' => 5.20,
                'v' => 5.42,
                'y' => 5.43,
                'k' => 5.46,
                'S' | 'F' => 5.51,
                '|' => 5.53,
                '?' => 5.56,
                'Z' => 5.62,
                'g' => 5.65,
                'a' => 5.75,
                'T' | 'Y' => 5.85,
                'E' => 5.86,
                'e' => 5.97,
                '_' | '+' | '*' | '=' | '#' | '<' | '>' | '^' | '~' => 5.98,
                '0'..='9' | '$' => 6.08,
                'P' => 6.20,
                'K' | 'X' => 6.23,
                'b' | 'd' | 'h' | 'n' | 'p' | 'q' | 'u' => 6.30,
                'o' => 6.33,
                'C' => 6.40,
                'R' => 6.52,
                'B' => 6.53,
                'V' => 6.81,
                'A' => 6.94,
                'D' => 7.38,
                'H' => 7.48,
                'G' => 7.57,
                'U' => 7.70,
                'N' => 7.75,
                'O' => 7.95,
                'Q' => 8.07,
                '&' => 8.19,
                'w' | '%' => 8.58,
                'm' => 9.59,
                'M' => 10.26,
                'W' => 10.68,
                '@' => 10.73,
                _ => 5.85,
            };
            base * (font_size / 12.0)
        }
        FontType::Inter => {
            let base = match ch {
                'i' | 'l' | '\'' | '|' | '!' | '.' | ',' | ';' | ':' | '`' => 3.20,
                'I' | 'j' => 3.30,
                ' ' | '\u{00A0}' => 3.50,
                't' | 'f' | 'r' | '(' | ')' | '[' | ']' | '{' | '}' | '-' | '_' | '+' | '*'
                | '=' => 4.30,
                'J' | 's' | 'z' => 5.20,
                'c' | 'v' | 'x' | '?' | '/' | '\\' => 5.70,
                'k' => 6.05,
                'e' => 6.20,
                'a' => 6.25,
                'b' | 'd' | 'g' | 'h' | 'n' | 'o' | 'p' | 'q' | 'u' | 'y' => 6.40,
                '0'..='9' => 6.80,
                'L' | 'S' => 6.40,
                'F' | 'E' | 'T' | 'Z' => 6.80,
                'P' | 'Y' => 7.20,
                'B' | 'K' | 'R' | 'V' | 'X' => 7.70,
                'A' | 'C' => 7.80,
                'G' | 'D' | 'U' => 8.20,
                'H' | 'N' | 'O' | 'Q' => 8.50,
                'w' => 9.00,
                '@' | '%' | '&' | '#' | '$' => 9.15,
                'm' => 9.90,
                'M' => 10.10,
                'W' => 11.10,
                _ => 6.30,
            };
            base * (font_size / 12.0)
        }
        FontType::SegoeUI => {
            let base = match ch {
                // Very narrow characters (~3.15px - 3.25px)
                'i' | 'l' | '\'' | '|' | '!' | '.' | ',' | ';' | ':' | '`' => 3.18,
                'I' | 'j' => 3.28,
                ' ' | '\u{00A0}' => 3.55,

                // Narrow characters (~4.35px - 5.15px)
                't' | 'f' | 'r' | '(' | ')' | '[' | ']' | '{' | '}' | '-' | '_' | '+' | '*'
                | '=' => 4.35,
                'J' | 's' | 'z' => 5.15,

                // Lowercase mid-width (~5.60px - 6.00px)
                'c' | 'v' | 'x' | '?' | '/' | '\\' => 5.65,
                'k' => 6.00,

                // Standard lowercase (~6.15px - 6.35px)
                'e' => 6.15,
                'a' => 6.22,
                'b' | 'd' | 'g' | 'h' | 'n' | 'o' | 'p' | 'q' | 'u' | 'y' => 6.35,

                // Standard digits (~6.85px)
                '0'..='9' => 6.85,

                // Narrow uppercase (~6.30px - 7.15px)
                'L' | 'S' => 6.30,
                'F' => 6.50,
                'E' | 'T' | 'Z' => 6.75,
                'P' | 'Y' => 7.15,

                // Standard uppercase (~7.65px - 8.45px)
                'B' | 'K' | 'R' | 'V' | 'X' => 7.65,
                'A' | 'C' => 7.75,
                'G' => 8.05,
                'D' | 'U' => 8.18,
                'H' | 'N' => 8.42,
                'O' | 'Q' => 8.48,

                // Wide characters (~8.95px - 11.00px)
                'w' => 8.95,
                '@' | '%' | '&' | '#' | '$' => 9.10,
                'm' => 9.80,
                'M' => 10.05,
                'W' => 11.00,

                // Default average
                _ => 6.28,
            };
            base * (font_size / 12.0)
        }
    }
}
