use crate::error::{DecodeError, EncodeError};

pub mod read_device_id;
pub mod read_primary_variable;
pub mod read_loop_current;
pub mod read_dynamic_vars;
pub mod write_polling_address;
pub mod read_loop_config;
pub mod read_dynamic_var_class;
pub mod read_device_vars;
pub mod read_unique_id_by_tag;
pub mod read_message;
pub mod read_tag_descriptor_date;
pub mod read_pv_transducer_info;
pub mod read_device_info;
pub mod read_final_assembly;
pub mod write_message;
pub mod write_tag_descriptor_date;
pub mod write_final_assembly;
pub mod read_long_tag;
pub mod write_long_tag;
pub mod reset_config_changed;
pub mod read_additional_status;

/// Trait for HART command requests.
pub trait CommandRequest {
    const COMMAND_NUMBER: u8;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError>;
}

/// Trait for HART command responses.
pub trait CommandResponse: Sized {
    const COMMAND_NUMBER: u8;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError>;
}

// --- Shared helpers for repeated encode/decode patterns ---

/// Decode a 24-bit big-endian unsigned integer from 3 bytes.
pub(crate) fn decode_u24_be(data: &[u8]) -> u32 {
    ((data[0] as u32) << 16) | ((data[1] as u32) << 8) | (data[2] as u32)
}

/// Encode a 24-bit big-endian unsigned integer into 3 bytes.
pub(crate) fn encode_u24_be(value: u32, buf: &mut [u8]) {
    buf[0] = ((value >> 16) & 0xFF) as u8;
    buf[1] = ((value >> 8) & 0xFF) as u8;
    buf[2] = (value & 0xFF) as u8;
}

/// Decode a HART device identity block (12 bytes, used by Command 0 and Command 11 responses).
///
/// Layout:
///   [0]     expansion_code
///   [1..2]  expanded_device_type (big-endian u16)
///   [3]     min_preamble_count
///   [4]     hart_revision
///   [5]     device_revision
///   [6]     software_revision
///   [7]     hw_rev_and_signaling (hw_revision = bits[7:3], physical_signaling = bits[2:0])
///   [8]     flags
///   [9..11] device_id (24-bit big-endian)
pub(crate) struct DeviceIdentity {
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

impl DeviceIdentity {
    pub fn decode(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 12 {
            return Err(DecodeError::BufferTooShort);
        }
        let hw_raw = data[7];
        Ok(DeviceIdentity {
            expansion_code: data[0],
            expanded_device_type: ((data[1] as u16) << 8) | (data[2] as u16),
            min_preamble_count: data[3],
            hart_revision: data[4],
            device_revision: data[5],
            software_revision: data[6],
            hardware_revision: hw_raw >> 3,
            physical_signaling: hw_raw & 0x07,
            flags: data[8],
            device_id: decode_u24_be(&data[9..12]),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u24_be_roundtrip() {
        let mut buf = [0u8; 3];
        encode_u24_be(0x112233, &mut buf);
        assert_eq!(buf, [0x11, 0x22, 0x33]);
        assert_eq!(decode_u24_be(&buf), 0x112233);
    }

    #[test]
    fn test_u24_be_zero() {
        let mut buf = [0u8; 3];
        encode_u24_be(0, &mut buf);
        assert_eq!(buf, [0x00, 0x00, 0x00]);
        assert_eq!(decode_u24_be(&buf), 0);
    }

    #[test]
    fn test_u24_be_max() {
        let mut buf = [0u8; 3];
        encode_u24_be(0x00FF_FFFF, &mut buf);
        assert_eq!(buf, [0xFF, 0xFF, 0xFF]);
        assert_eq!(decode_u24_be(&buf), 0x00FF_FFFF);
    }

    #[test]
    fn test_device_identity_decode() {
        let data = [
            0xFE, 0x1A, 0x2B, 0x05, 0x07, 0x01, 0x03, 0x04, 0x00, 0x11, 0x22, 0x33,
        ];
        let id = DeviceIdentity::decode(&data).unwrap();
        assert_eq!(id.expansion_code, 0xFE);
        assert_eq!(id.expanded_device_type, 0x1A2B);
        assert_eq!(id.min_preamble_count, 5);
        assert_eq!(id.hart_revision, 7);
        assert_eq!(id.device_revision, 1);
        assert_eq!(id.software_revision, 3);
        assert_eq!(id.hardware_revision, 0);
        assert_eq!(id.physical_signaling, 4);
        assert_eq!(id.flags, 0);
        assert_eq!(id.device_id, 0x112233);
    }

    #[test]
    fn test_device_identity_decode_too_short() {
        let data = [0u8; 11];
        assert_eq!(
            DeviceIdentity::decode(&data).err(),
            Some(DecodeError::BufferTooShort)
        );
    }
}
