//! Command 0 — Read Unique Identifier (Device ID)

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_DEVICE_ID;
use crate::error::{DecodeError, EncodeError};

/// Command 0 request: no data payload.
#[derive(Debug, Clone)]
pub struct ReadDeviceIdRequest;

/// Command 0 response: device identification fields.
///
/// Layout (12 bytes):
///   - `[0]`     `expansion_code`
///   - `[1..2]`  `expanded_device_type` (big-endian u16)
///   - `[3]`     `min_preamble_count`
///   - `[4]`     `hart_revision`
///   - `[5]`     `device_revision`
///   - `[6]`     `software_revision`
///   - `[7]`     `hw_rev_and_signaling` (`hardware_revision` = bits\[7:3\], `physical_signaling` = bits\[2:0\])
///   - `[8]`     `flags`
///   - `[9..11]` `device_id` (24-bit big-endian)
#[derive(Debug, Clone, PartialEq)]
pub struct ReadDeviceIdResponse {
    /// HART expansion code (0xFE = expanded).
    pub expansion_code: u8,
    /// Expanded device type identifier.
    pub expanded_device_type: u16,
    /// Minimum preamble count the device expects.
    pub min_preamble_count: u8,
    /// HART protocol revision supported by the device.
    pub hart_revision: u8,
    /// Device-specific revision number.
    pub device_revision: u8,
    /// Device software revision number.
    pub software_revision: u8,
    /// Hardware revision (upper 5 bits of byte 7).
    pub hardware_revision: u8,
    /// Physical signaling code (lower 3 bits of byte 7).
    pub physical_signaling: u8,
    /// Device flags byte.
    pub flags: u8,
    /// 24-bit unique device identifier.
    pub device_id: u32,
}

impl CommandRequest for ReadDeviceIdRequest {
    const COMMAND_NUMBER: u8 = READ_DEVICE_ID;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for ReadDeviceIdResponse {
    const COMMAND_NUMBER: u8 = READ_DEVICE_ID;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        let id = super::DeviceIdentity::decode(data)?;
        Ok(ReadDeviceIdResponse {
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
        let req = ReadDeviceIdRequest;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
        assert_eq!(ReadDeviceIdRequest::COMMAND_NUMBER, 0);
    }

    #[test]
    fn test_cmd0_response_decode() {
        let data = [
            0xFE, 0x1A, 0x2B, 0x05, 0x07, 0x01, 0x03, 0x04, 0x00, 0x11, 0x22, 0x33,
        ];

        let resp = ReadDeviceIdResponse::decode_data(&data).unwrap();
        assert_eq!(resp.expansion_code, 0xFE);
        assert_eq!(resp.expanded_device_type, 0x1A2B);
        assert_eq!(resp.min_preamble_count, 5);
        assert_eq!(resp.hart_revision, 7);
        assert_eq!(resp.device_revision, 1);
        assert_eq!(resp.software_revision, 3);
        assert_eq!(resp.hardware_revision, 0);
        assert_eq!(resp.physical_signaling, 4);
        assert_eq!(resp.flags, 0);
        assert_eq!(resp.device_id, 0x112233);
    }

    #[test]
    fn test_cmd0_response_too_short() {
        let data = [0u8; 11];
        assert_eq!(
            ReadDeviceIdResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd0_command_number() {
        assert_eq!(ReadDeviceIdRequest::COMMAND_NUMBER, 0);
        assert_eq!(ReadDeviceIdResponse::COMMAND_NUMBER, 0);
    }
}
