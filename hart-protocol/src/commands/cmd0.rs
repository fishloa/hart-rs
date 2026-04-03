//! Command 0 — Read Unique Identifier (Device ID)

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_DEVICE_ID;
use crate::error::{DecodeError, EncodeError};

/// Command 0 request: no data payload.
#[derive(Debug, Clone)]
pub struct Cmd0Request;

/// Command 0 response: device identification fields.
///
/// Layout (12 bytes):
///   [0]     expansion_code
///   [1..2]  expanded_device_type (big-endian u16)
///   [3]     min_preamble_count
///   [4]     hart_revision
///   [5]     device_revision
///   [6]     software_revision
///   [7]     hw_rev_and_signaling (hw_revision = bits[7:3], physical_signaling = bits[2:0])
///   [8]     flags
///   [9..11] device_id (24-bit big-endian)
#[derive(Debug, Clone, PartialEq)]
pub struct Cmd0Response {
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

impl CommandRequest for Cmd0Request {
    const COMMAND_NUMBER: u8 = READ_DEVICE_ID;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for Cmd0Response {
    const COMMAND_NUMBER: u8 = READ_DEVICE_ID;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        let id = super::DeviceIdentity::decode(data)?;
        Ok(Cmd0Response {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd0_request_encodes_no_data() {
        let req = Cmd0Request;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
        assert_eq!(Cmd0Request::COMMAND_NUMBER, 0);
    }

    #[test]
    fn test_cmd0_response_decode() {
        // From RESP_CMD0_LONG test vector (after stripping 2 status bytes):
        // expansion=0xFE, manufacturer_id(exp_type_hi)=0x1A, device_type(exp_type_lo)=0x2B,
        // preambles=5, hart_rev=7, dev_rev=1, sw_rev=3, hw_rev_raw=0x04, flags=0, device_id=0x112233
        // hw_revision = 0x04 >> 3 = 0, physical_signaling = 0x04 & 0x07 = 4
        let data = [
            0xFE, // expansion_code
            0x1A, // expanded_device_type high
            0x2B, // expanded_device_type low
            0x05, // min_preamble_count
            0x07, // hart_revision
            0x01, // device_revision
            0x03, // software_revision
            0x04, // hw_rev_and_signaling
            0x00, // flags
            0x11, 0x22, 0x33, // device_id
        ];

        let resp = Cmd0Response::decode_data(&data).unwrap();
        assert_eq!(resp.expansion_code, 0xFE);
        assert_eq!(resp.expanded_device_type, 0x1A2B);
        assert_eq!(resp.min_preamble_count, 5);
        assert_eq!(resp.hart_revision, 7);
        assert_eq!(resp.device_revision, 1);
        assert_eq!(resp.software_revision, 3);
        assert_eq!(resp.hardware_revision, 0); // 0x04 >> 3 = 0
        assert_eq!(resp.physical_signaling, 4); // 0x04 & 0x07 = 4
        assert_eq!(resp.flags, 0);
        assert_eq!(resp.device_id, 0x112233);
    }

    #[test]
    fn test_cmd0_response_too_short() {
        let data = [0u8; 11]; // needs 12
        assert_eq!(
            Cmd0Response::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd0_command_number() {
        assert_eq!(Cmd0Request::COMMAND_NUMBER, 0);
        assert_eq!(Cmd0Response::COMMAND_NUMBER, 0);
    }
}
