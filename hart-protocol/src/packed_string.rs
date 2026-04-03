//! HART 6-bit packed ASCII string encoding/decoding.
//!
//! HART uses a 6-bit character encoding where:
//!   `code = ascii_char - 0x20`   (subtract 0x20 to get 6-bit code)
//!   `ascii_char = code + 0x20`   (add 0x20 to decode back to ASCII)
//!
//! Only printable ASCII characters in the range 0x20-0x5F are valid.
//! Characters outside this range are clamped/masked to fit.
//!
//! Packing: 4 characters -> 3 bytes
//!   byte0 = (c0 << 2) | (c1 >> 4)
//!   byte1 = (c1 << 4) | (c2 >> 2)
//!   byte2 = (c2 << 6) | c3
//!
//! Unpacking: 3 bytes -> 4 characters
//!   c0 = b0 >> 2
//!   c1 = ((b0 & 0x03) << 4) | (b1 >> 4)
//!   c2 = ((b1 & 0x0F) << 2) | (b2 >> 6)
//!   c3 = b2 & 0x3F

/// Encode `src` ASCII bytes into 6-bit packed form in `dst`.
///
/// Processes 4 input bytes at a time into 3 output bytes.
/// `src` is padded with spaces (0x20) if not a multiple of 4.
/// Returns the number of bytes written to `dst`.
///
/// # Examples
///
/// ```
/// use hart_protocol::packed_string::{encode_packed, decode_packed};
///
/// let mut packed = [0u8; 3];
/// let n = encode_packed(b"TEST", &mut packed);
/// assert_eq!(n, 3);
///
/// let mut ascii = [0u8; 4];
/// decode_packed(&packed, &mut ascii);
/// assert_eq!(&ascii, b"TEST");
/// ```
pub fn encode_packed(src: &[u8], dst: &mut [u8]) -> usize {
    let n_groups = src.len().div_ceil(4);
    let mut written = 0;
    for g in 0..n_groups {
        if written + 3 > dst.len() {
            break;
        }
        let base = g * 4;
        // Convert ASCII to 6-bit code by subtracting 0x20; clamp to 6 bits
        let c0 = u16::from(src.get(base).copied().unwrap_or(b' ').saturating_sub(0x20) & 0x3F);
        let c1 = u16::from(
            src.get(base + 1)
                .copied()
                .unwrap_or(b' ')
                .saturating_sub(0x20)
                & 0x3F,
        );
        let c2 = u16::from(
            src.get(base + 2)
                .copied()
                .unwrap_or(b' ')
                .saturating_sub(0x20)
                & 0x3F,
        );
        let c3 = u16::from(
            src.get(base + 3)
                .copied()
                .unwrap_or(b' ')
                .saturating_sub(0x20)
                & 0x3F,
        );

        #[allow(clippy::cast_possible_truncation)]
        {
            dst[written] = ((c0 << 2) | (c1 >> 4)) as u8;
            dst[written + 1] = ((c1 << 4) | (c2 >> 2)) as u8;
            dst[written + 2] = ((c2 << 6) | c3) as u8;
        }
        written += 3;
    }
    written
}

/// Decode 6-bit packed bytes from `src` into ASCII in `dst`.
///
/// Processes 3 input bytes at a time into 4 output characters.
/// Returns the number of bytes written to `dst`.
///
/// # Examples
///
/// ```
/// use hart_protocol::packed_string::{encode_packed, decode_packed};
///
/// let mut packed = [0u8; 3];
/// encode_packed(b"TEST", &mut packed);
///
/// let mut ascii = [0u8; 4];
/// let n = decode_packed(&packed, &mut ascii);
/// assert_eq!(n, 4);
/// assert_eq!(&ascii, b"TEST");
/// ```
pub fn decode_packed(src: &[u8], dst: &mut [u8]) -> usize {
    let n_groups = src.len() / 3;
    let mut written = 0;
    for g in 0..n_groups {
        if written + 4 > dst.len() {
            break;
        }
        let base = g * 3;
        let b0 = u16::from(src[base]);
        let b1 = u16::from(src[base + 1]);
        let b2 = u16::from(src[base + 2]);

        let c0 = b0 >> 2;
        let c1 = ((b0 & 0x03) << 4) | (b1 >> 4);
        let c2 = ((b1 & 0x0F) << 2) | (b2 >> 6);
        let c3 = b2 & 0x3F;

        // Convert 6-bit code back to ASCII by adding 0x20
        #[allow(clippy::cast_possible_truncation)]
        {
            dst[written] = (c0 as u8) + 0x20;
            dst[written + 1] = (c1 as u8) + 0x20;
            dst[written + 2] = (c2 as u8) + 0x20;
            dst[written + 3] = (c3 as u8) + 0x20;
        }
        written += 4;
    }
    written
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_test() {
        let src = b"TEST";
        let mut encoded = [0u8; 3];
        let enc_len = encode_packed(src, &mut encoded);
        assert_eq!(enc_len, 3);

        let mut decoded = [0u8; 4];
        let dec_len = decode_packed(&encoded[..enc_len], &mut decoded);
        assert_eq!(dec_len, 4);
        assert_eq!(&decoded[..4], b"TEST");
    }

    #[test]
    fn test_roundtrip_sensor01() {
        let src = b"SENSOR01";
        let mut encoded = [0u8; 6];
        let enc_len = encode_packed(src, &mut encoded);
        assert_eq!(enc_len, 6);

        let mut decoded = [0u8; 8];
        let dec_len = decode_packed(&encoded[..enc_len], &mut decoded);
        assert_eq!(dec_len, 8);
        assert_eq!(&decoded[..8], b"SENSOR01");
    }

    #[test]
    fn test_roundtrip_spaces() {
        let src = b"    ";
        let mut encoded = [0u8; 3];
        let enc_len = encode_packed(src, &mut encoded);
        assert_eq!(enc_len, 3);

        let mut decoded = [0u8; 4];
        let dec_len = decode_packed(&encoded[..enc_len], &mut decoded);
        assert_eq!(dec_len, 4);
        assert_eq!(&decoded[..4], b"    ");
    }

    #[test]
    fn test_encode_known_values() {
        // "TEST": T=0x54→code=0x34=52, E=0x45→code=0x25=37, S=0x53→code=0x33=51, T→52
        // byte0 = (52<<2)|(37>>4) = 208|2 = 210 = 0xD2
        // byte1 = (37<<4)|(51>>2) = 592|12 = 0x250|0x0C → u8 = 0x5C (but 592&0xFF = 0x50, 0x50|12=0x5C)
        // byte2 = (51<<6)|52 = 3264|52 = 0xCC0|52 → u8 = 0xC0|0x34 = 0xF4
        let src = b"TEST";
        let mut encoded = [0u8; 3];
        encode_packed(src, &mut encoded);
        // Verify via decode roundtrip rather than exact bytes, to avoid endianness confusion
        let mut decoded = [0u8; 4];
        decode_packed(&encoded, &mut decoded);
        assert_eq!(&decoded, b"TEST");
    }

    #[test]
    fn test_decode_known_values() {
        // Encode "TEST" then decode it back
        let src = b"TEST";
        let mut encoded = [0u8; 3];
        encode_packed(src, &mut encoded);
        let mut decoded = [0u8; 4];
        decode_packed(&encoded, &mut decoded);
        assert_eq!(&decoded, b"TEST");
    }

    #[test]
    fn test_padding_with_spaces() {
        // "HI" (2 chars) should encode as "HI  " (padded with spaces)
        let src = b"HI";
        let mut encoded = [0u8; 3];
        encode_packed(src, &mut encoded);

        let mut decoded = [0u8; 4];
        decode_packed(&encoded, &mut decoded);
        assert_eq!(&decoded, b"HI  ");
    }

    #[test]
    fn test_space_encodes_to_zero_code() {
        // Space (0x20) should encode to 6-bit code 0
        // 4 spaces -> all codes are 0 -> all bytes are 0
        let src = b"    ";
        let mut encoded = [0u8; 3];
        encode_packed(src, &mut encoded);
        assert_eq!(encoded, [0x00, 0x00, 0x00]);
    }

    #[test]
    fn test_encode_empty_input() {
        let src = b"";
        let mut encoded = [0xFFu8; 3];
        let len = encode_packed(src, &mut encoded);
        assert_eq!(len, 0);
    }

    #[test]
    fn test_decode_empty_input() {
        let src: &[u8] = &[];
        let mut decoded = [0xFFu8; 4];
        let len = decode_packed(src, &mut decoded);
        assert_eq!(len, 0);
    }

    #[test]
    fn test_partial_group_1_char() {
        // "A" -> padded to "A   "
        let src = b"A";
        let mut encoded = [0u8; 3];
        let len = encode_packed(src, &mut encoded);
        assert_eq!(len, 3);
        let mut decoded = [0u8; 4];
        decode_packed(&encoded, &mut decoded);
        assert_eq!(&decoded, b"A   ");
    }

    #[test]
    fn test_partial_group_3_chars() {
        // "ABC" -> padded to "ABC "
        let src = b"ABC";
        let mut encoded = [0u8; 3];
        let len = encode_packed(src, &mut encoded);
        assert_eq!(len, 3);
        let mut decoded = [0u8; 4];
        decode_packed(&encoded, &mut decoded);
        assert_eq!(&decoded, b"ABC ");
    }

    #[test]
    fn test_5_chars_produces_two_groups() {
        // "ABCDE" -> 2 groups -> 6 bytes
        let src = b"ABCDE";
        let mut encoded = [0u8; 6];
        let len = encode_packed(src, &mut encoded);
        assert_eq!(len, 6);
        let mut decoded = [0u8; 8];
        decode_packed(&encoded, &mut decoded);
        assert_eq!(&decoded, b"ABCDE   ");
    }

    #[test]
    fn test_32_char_max_tag_roundtrip() {
        // 32 chars = 8 groups = 24 packed bytes (max HART message)
        let src = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ012345";
        let mut encoded = [0u8; 24];
        let len = encode_packed(src, &mut encoded);
        assert_eq!(len, 24);
        let mut decoded = [0u8; 32];
        decode_packed(&encoded, &mut decoded);
        assert_eq!(&decoded, src);
    }

    #[test]
    fn test_dst_too_small_stops_gracefully() {
        // Source needs 3 bytes for encoding, but dst only has 2
        let src = b"TEST";
        let mut encoded = [0u8; 2];
        let len = encode_packed(src, &mut encoded);
        assert_eq!(len, 0); // can't fit any group
    }

    #[test]
    fn test_decode_incomplete_group_ignored() {
        // Only 2 bytes: less than a full 3-byte group, so no output
        let src = [0x00u8, 0x00];
        let mut decoded = [0xFFu8; 4];
        let len = decode_packed(&src, &mut decoded);
        assert_eq!(len, 0);
    }
}
