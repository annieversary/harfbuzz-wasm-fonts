use harfbuzz_wasm::{Font, Glyph, GlyphBuffer};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn shape(
    _shape_plan: u32,
    font_ref: u32,
    buf_ref: u32,
    _features: u32,
    _num_features: u32,
) -> i32 {
    let font = Font::from_ref(font_ref);
    let mut buffer = GlyphBuffer::from_ref(buf_ref);

    let mut out = vec![];

    let mut digits = vec![];
    for item in buffer.glyphs.iter() {
        if let Some(number) = unicode_codepoint_to_number(item.codepoint) {
            digits.push(number);
        } else {
            // process digits, add them as roman numerals instead of the actual glyphs
            process_digits(&mut digits, &mut out);

            out.push(*item);
        }
    }

    // also handle non empty digits here, otherwise numbers at the end of the string won't work
    process_digits(&mut digits, &mut out);

    // fix characters
    for item in out.iter_mut() {
        let is_overline = item.codepoint == 0x305;

        // Map character to glyph
        item.codepoint = font.get_glyph(item.codepoint, 0);

        // Set advance
        item.x_advance = if is_overline {
            // overline doesn't move forward,
            // since we want the next character at the same spot
            0
        } else {
            font.get_glyph_h_advance(item.codepoint)
        };

        // we want overlines to be a bit higher
        item.y_offset = if is_overline { 130 } else { 0 };
    }

    buffer.glyphs = out;

    // Buffer is written back to HB on drop
    1
}

fn unicode_codepoint_to_number(unicode: u32) -> Option<u8> {
    if (0x30..=0x39).contains(&unicode) {
        Some((unicode - 0x30) as u8)
    } else {
        None
    }
}

fn process_digits(digits: &mut Vec<u8>, out: &mut Vec<Glyph>) {
    // if we had some numbers, and now we're in a non-number now
    // it means we gotta render the numbers
    if !digits.is_empty() {
        // turn digits into a single number...
        let number = digits_to_number(digits);
        // ...then turn number to roman numerals string...
        let roman = number_to_roman_numeral(number);
        let roman_glyphs = string_to_glyphs(&roman);
        // ...and add the roman glyphs to the output
        out.extend_from_slice(&roman_glyphs);

        digits.clear();
    }
}

fn digits_to_number(digits: &[u8]) -> u64 {
    digits.iter().rev().enumerate().fold(0, |acc, (idx, num)| {
        acc + (*num as u64) * 10u64.pow(idx as u32)
    })
}

fn string_to_glyphs(string: &str) -> Vec<Glyph> {
    string
        .chars()
        .enumerate()
        .map(|(ix, x)| Glyph {
            codepoint: if x == '_' { 0x305 } else { x as u32 },
            flags: 0,
            x_advance: 0,
            y_advance: 0,
            cluster: if x == '_' { ix + 1 } else { ix } as u32,
            x_offset: 0,
            y_offset: 0,
        })
        .collect()
}

fn number_to_roman_numeral(mut number: u64) -> String {
    let letters = [
        (1_000_000, "_M"),
        (900_000, "_C_M"),
        (500_000, "_D"),
        (400_000, "_C_D"),
        (100_000, "_C"),
        (90_000, "_X_C"),
        (50_000, "_L"),
        (40_000, "_X_L"),
        (10_000, "_X"),
        (9_000, "_I_X"),
        (5_000, "_V"),
        (4_000, "_I_V"),
        (1_000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    let mut result = String::new();

    for (value, symbol) in letters {
        while number >= value {
            result.push_str(symbol);
            number -= value;
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_digits_to_number() {
        fn test(digits: &[u8], expected: u64) -> bool {
            digits_to_number(digits) == expected
        }

        assert!(test(&[1], 1));
        assert!(test(&[2], 2));

        assert!(test(&[1, 1, 1], 111));
        assert!(test(&[1, 2, 3], 123));
        assert!(test(&[3, 2, 1], 321));

        assert!(test(&[3, 2, 1, 4, 5, 6], 321456));

        assert!(test(&[], 0));
    }

    #[test]
    fn test_number_to_roman_numeral() {
        assert_eq!("I", number_to_roman_numeral(1));
        assert_eq!("II", number_to_roman_numeral(2));
        assert_eq!("IX", number_to_roman_numeral(9));
        assert_eq!("XI", number_to_roman_numeral(11));

        assert_eq!("CXXI", number_to_roman_numeral(121));

        assert_eq!("MMCCCXXI", number_to_roman_numeral(2321));

        assert_eq!("_XMMCCCXXI", number_to_roman_numeral(12321));
    }
}
