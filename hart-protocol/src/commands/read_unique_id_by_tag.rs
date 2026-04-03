//! Command 11 — Read Unique Identifier by Tag

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_UNIQUE_ID_BY_TAG;
use crate::error::{DecodeError, EncodeError};
use crate::packed_string::{decode_packed, encode_packed};

/// Command 11 request: 8-char tag encoded as 6 packed bytes.
#[derive(Debug, Clone)]
pub struct Cmd11Request {
    /// 8-character ASCII tag (padded with spaces if shorter).
    pub tag: [u8; 8],
}

/// Command 11 response: device identification fields (same layout as Command 0).
///
/// Layout (12 bytes):
///   [0]     expansion_code
///   [1..2]  expanded_device_type (big-endian u16)
///   [3]     min_preamble_count
///   [4]     hart_revision
///   [5]     device_revision
///   [6]     software_revision
///   [7]     hw_rev_and_signaling
///   [8]     flags
///   [9..11] device_id (24-bit big-endian)
#[derive(Debug, Clone, PartialEq)]
pub struct Cmd11Response {
    pub expansion_code: u8,
    pub expanded_device_type: u16,
    pub min_preamble_count: u8,
    pub hart_revision: u8,
    pub device_revision: u8,
    pub software_revision: u8,
    pub hardware_revision: u8,
    pub physical_signaling: u8,
    pub flags: u8,
    pub device_id: u32,
}

impl CommandRequest for Cmd11Request {
    const COMMAND_NUMBER: u8 = READ_UNIQUE_ID_BY_TAG;

    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 6 {
            return Err(EncodeError::BufferTooSmall);
        }
        encode_packed(&self.tag, &mut buf[..6]);
        Ok(6)
    }
}

impl CommandResponse for Cmd11Response {
    const COMMAND_NUMBER: u8 = READ_UNIQUE_ID_BY_TAG;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        let id = super::DeviceIdentity::decode(data)?;
        Ok(Cmd11Response {
            expansion_code: id.expansion_code,
            expanded_device_type: id.expanded_device_type,
            min_preamble_count: id.min_preamble_count,
            hart_revision: id.hart_revision,
            device_revision: id.device_revision,
            software_revision: id.software_revision,
            hardware_revision: id.hardware_revision,
            physical_signaling: id.physical_signaling,
            flags: id.flags,
            device_id: id.device_id,
        })
    }
}

/// Decode a 6-byte packed tag into an 8-byte ASCII buffer.
pub fn decode_tag(packed: &[u8; 6]) -> [u8; 8] {
    let mut tag = [b' '; 8];
    decode_packed(packed, &mut tag);
    tag
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd11_command_number() {
        assert_eq!(Cmd11Request::COMMAND_NUMBER, 11);
        assert_eq!(Cmd11Response::COMMAND_NUMBER, 11);
    }

    #[test]
    fn test_cmd11_request_encode() {
        let mut tag = [b' '; 8];
        tag[..4].copy_from_slice(b"TEST");
        let req = Cmd11Request { tag };
        let mut buf = [0u8; 6];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 6);
        // Verify it decodes back to "TEST    "
        let mut decoded = [0u8; 8];
        decode_packed(&buf[..6], &mut decoded);
        assert_eq!(&decoded, b"TEST    ");
    }

    #[test]
    fn test_cmd11_request_buffer_too_small() {
        let req = Cmd11Request { tag: [b' '; 8] };
        let mut buf = [0u8; 3]; // too small
        assert_eq!(req.encode_data(&mut buf), Err(EncodeError::BufferTooSmall));
    }

    #[test]
    fn test_cmd11_response_decode() {
        let data = [
            0xFE, 0x1A, 0x2B, 0x05, 0x07, 0x01, 0x03, 0x04, 0x00, 0x11, 0x22, 0x33,
        ];
        let resp = Cmd11Response::decode_data(&data).unwrap();
        assert_eq!(resp.expansion_code, 0xFE);
        assert_eq!(resp.expanded_device_type, 0x1A2B);
        assert_eq!(resp.device_id, 0x112233);
    }

    #[test]
    fn test_cmd11_response_too_short() {
        let data = [0u8; 11];
        assert_eq!(
            Cmd11Response::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_decode_tag() {
        let mut tag_src = [b' '; 8];
        tag_src[..8].copy_from_slice(b"SENSOR01");
        let mut packed = [0u8; 6];
        encode_packed(&tag_src, &mut packed);
        let decoded = decode_tag(&packed);
        assert_eq!(&decoded, b"SENSOR01");
    }
}
