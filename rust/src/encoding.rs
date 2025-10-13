use std::{collections::HashMap, sync::LazyLock};

use crate::error::Ksj2GpError;

/// Unicode code points for each byte value in code page 437.
const CP437_UNICODE: [u16; 256] = [
    0x0000, 0x0001, 0x0002, 0x0003, 0x0004, 0x0005, 0x0006, 0x0007, 0x0008, 0x0009, 0x000a, 0x000b,
    0x000c, 0x000d, 0x000e, 0x000f, 0x0010, 0x0011, 0x0012, 0x0013, 0x0014, 0x0015, 0x0016, 0x0017,
    0x0018, 0x0019, 0x001a, 0x001b, 0x001c, 0x001d, 0x001e, 0x001f, 0x0020, 0x0021, 0x0022, 0x0023,
    0x0024, 0x0025, 0x0026, 0x0027, 0x0028, 0x0029, 0x002a, 0x002b, 0x002c, 0x002d, 0x002e, 0x002f,
    0x0030, 0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037, 0x0038, 0x0039, 0x003a, 0x003b,
    0x003c, 0x003d, 0x003e, 0x003f, 0x0040, 0x0041, 0x0042, 0x0043, 0x0044, 0x0045, 0x0046, 0x0047,
    0x0048, 0x0049, 0x004a, 0x004b, 0x004c, 0x004d, 0x004e, 0x004f, 0x0050, 0x0051, 0x0052, 0x0053,
    0x0054, 0x0055, 0x0056, 0x0057, 0x0058, 0x0059, 0x005a, 0x005b, 0x005c, 0x005d, 0x005e, 0x005f,
    0x0060, 0x0061, 0x0062, 0x0063, 0x0064, 0x0065, 0x0066, 0x0067, 0x0068, 0x0069, 0x006a, 0x006b,
    0x006c, 0x006d, 0x006e, 0x006f, 0x0070, 0x0071, 0x0072, 0x0073, 0x0074, 0x0075, 0x0076, 0x0077,
    0x0078, 0x0079, 0x007a, 0x007b, 0x007c, 0x007d, 0x007e, 0x007f, 0x00c7, 0x00fc, 0x00e9, 0x00e2,
    0x00e4, 0x00e0, 0x00e5, 0x00e7, 0x00ea, 0x00eb, 0x00e8, 0x00ef, 0x00ee, 0x00ec, 0x00c4, 0x00c5,
    0x00c9, 0x00e6, 0x00c6, 0x00f4, 0x00f6, 0x00f2, 0x00fb, 0x00f9, 0x00ff, 0x00d6, 0x00dc, 0x00a2,
    0x00a3, 0x00a5, 0x20a7, 0x0192, 0x00e1, 0x00ed, 0x00f3, 0x00fa, 0x00f1, 0x00d1, 0x00aa, 0x00ba,
    0x00bf, 0x2310, 0x00ac, 0x00bd, 0x00bc, 0x00a1, 0x00ab, 0x00bb, 0x2591, 0x2592, 0x2593, 0x2502,
    0x2524, 0x2561, 0x2562, 0x2556, 0x2555, 0x2563, 0x2551, 0x2557, 0x255d, 0x255c, 0x255b, 0x2510,
    0x2514, 0x2534, 0x252c, 0x251c, 0x2500, 0x253c, 0x255e, 0x255f, 0x255a, 0x2554, 0x2569, 0x2566,
    0x2560, 0x2550, 0x256c, 0x2567, 0x2568, 0x2564, 0x2565, 0x2559, 0x2558, 0x2552, 0x2553, 0x256b,
    0x256a, 0x2518, 0x250c, 0x2588, 0x2584, 0x258c, 0x2590, 0x2580, 0x03b1, 0x00df, 0x0393, 0x03c0,
    0x03a3, 0x03c3, 0x00b5, 0x03c4, 0x03a6, 0x0398, 0x03a9, 0x03b4, 0x221e, 0x03c6, 0x03b5, 0x2229,
    0x2261, 0x00b1, 0x2265, 0x2264, 0x2320, 0x2321, 0x00f7, 0x2248, 0x00b0, 0x2219, 0x00b7, 0x221a,
    0x207f, 0x00b2, 0x25a0, 0x00a0,
];

/// Alternate glyphs commonly associated with control bytes in the CP437 table.
const CP437_ALT_GLYPHS: &[(char, u8)] = &[
    ('\u{263A}', 0x01), // ☺
    ('\u{263B}', 0x02), // ☻
    ('\u{2665}', 0x03), // ♥
    ('\u{2666}', 0x04), // ♦
    ('\u{2663}', 0x05), // ♣
    ('\u{2660}', 0x06), // ♠
    ('\u{2022}', 0x07), // •
    ('\u{25D8}', 0x08), // ◘
    ('\u{25CB}', 0x09), // ○
    ('\u{25D9}', 0x0A), // ◙
    ('\u{2642}', 0x0B), // ♂
    ('\u{2640}', 0x0C), // ♀
    ('\u{266A}', 0x0D), // ♪
    ('\u{266B}', 0x0E), // ♫
    ('\u{263C}', 0x0F), // ☼
    ('\u{25BA}', 0x10), // ►
    ('\u{25C4}', 0x11), // ◄
    ('\u{2195}', 0x12), // ↕
    ('\u{203C}', 0x13), // ‼
    ('\u{00B6}', 0x14), // ¶
    ('\u{00A7}', 0x15), // §
    ('\u{25AC}', 0x16), // ▬
    ('\u{21A8}', 0x17), // ↨
    ('\u{2191}', 0x18), // ↑
    ('\u{2193}', 0x19), // ↓
    ('\u{2192}', 0x1A), // →
    ('\u{2190}', 0x1B), // ←
    ('\u{221F}', 0x1C), // ∟
    ('\u{2194}', 0x1D), // ↔
    ('\u{25B2}', 0x1E), // ▲
    ('\u{25BC}', 0x1F), // ▼
    ('\u{2302}', 0x7F), // ⌂
];

fn cp437_map() -> &'static HashMap<char, u8> {
    static CP437_MAP: LazyLock<HashMap<char, u8>> = LazyLock::new(|| {
        let mut map = HashMap::with_capacity(CP437_UNICODE.len() + CP437_ALT_GLYPHS.len());
        for (byte, code_point) in CP437_UNICODE.iter().enumerate() {
            if let Some(ch) = char::from_u32(*code_point as u32) {
                map.insert(ch, byte as u8);
            }
        }
        for &(glyph, byte) in CP437_ALT_GLYPHS {
            map.insert(glyph, byte);
        }
        map
    });
    LazyLock::force(&CP437_MAP)
}

fn cp437_revmap() -> &'static HashMap<u8, char> {
    static CP437_REVMAP: LazyLock<HashMap<u8, char>> = LazyLock::new(|| {
        let mut map = HashMap::with_capacity(CP437_UNICODE.len());
        for (byte, code_point) in CP437_UNICODE.iter().enumerate() {
            if let Some(ch) = char::from_u32(*code_point as u32) {
                map.insert(byte as u8, ch);
            }
        }
        map
    });
    LazyLock::force(&CP437_REVMAP)
}

/// Decode CP437-encoded CP932 to UTF-8
pub fn decode_cp437cp932_to_utf8(input: &str) -> Result<String, Ksj2GpError> {
    let map = cp437_map();
    let mut cp932_bytes = Vec::with_capacity(input.len());

    for ch in input.chars() {
        if let Some(&byte) = map.get(&ch) {
            cp932_bytes.push(byte);
        } else {
            // If the input byte is beyond CP437 range, it probably means this is UTF-8
            return Ok(input.to_string());
        }
    }

    let (out, _, error) = encoding_rs::SHIFT_JIS.decode(&cp932_bytes);

    if error {
        return Err("Failed to decode from CP932".into());
    }

    Ok(out.to_string())
}

/// Encode UTF-8 to CP437-encoded CP932
pub fn encode_utf8_to_cp437cp932(input: &str) -> Result<String, Ksj2GpError> {
    let (cp932_bytes, _, error) = encoding_rs::SHIFT_JIS.encode(input);
    if error {
        return Err("Failed to encode to CP932".into());
    }

    let map = cp437_revmap();
    let mut cp437_chars = String::with_capacity(cp932_bytes.len());

    for b in cp932_bytes.iter() {
        if let Some(&byte) = map.get(b) {
            cp437_chars.push(byte);
        } else {
            return Err("Failed to encode to CP437".into());
        }
    }

    Ok(cp437_chars.to_string())
}

#[cfg(test)]
mod tests {
    use super::decode_cp437cp932_to_utf8;

    #[test]
    fn encodes_ascii_text() {
        let result = decode_cp437cp932_to_utf8("Hello, world!").unwrap();
        assert_eq!(result, "Hello, world!");
    }

    #[test]
    fn encodes_extended_characters() {
        let result = decode_cp437cp932_to_utf8("ÄDûyÄs").unwrap();
        assert_eq!(result, "札幌市");
    }

    #[test]
    fn skip_utf8_characters() {
        let result = decode_cp437cp932_to_utf8("札幌市").unwrap();
        assert_eq!(result, "札幌市");
    }
}
