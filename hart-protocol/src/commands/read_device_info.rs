//! Command 15 — Read Device Information

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_DEVICE_INFO;
use crate::error::{DecodeError, EncodeError};
use crate::units::UnitCode;

/// Command 15 request: no data payload.
#[derive(Debug, Clone)]
pub struct ReadDeviceInfoRequest;

/// Command 15 response: PV range, transfer function, and device info.
///
/// Layout (17 bytes):
///   0:     `pv_alarm_selection`
///   1:     `pv_transfer_function`
///   2:     `pv_unit` (unit code)
///   3..6:  `upper_range_value` (f32 big-endian)
///   7..10: `lower_range_value` (f32 big-endian)
///   11..14: `damping_value` (f32 big-endian)
///   15:    `write_protect_code`
///   16:    `private_label_distributor_code`
#[derive(Debug, Clone, PartialEq)]
pub struct ReadDeviceInfoResponse {
    /// PV alarm selection code.
    pub pv_alarm_selection: u8,
    /// PV transfer function code.
    pub pv_transfer_function: u8,
    /// PV engineering unit code.
    pub pv_unit: UnitCode,
    /// Upper range value.
    pub upper_range: f32,
    /// Lower range value.
    pub lower_range: f32,
    /// Damping value in seconds.
    pub damping: f32,
    /// Write protect code.
    pub write_protect: u8,
    /// Private label distributor code.
    pub private_label_distributor: u8,
}

impl CommandRequest for ReadDeviceInfoRequest {
    const COMMAND_NUMBER: u8 = READ_DEVICE_INFO;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for ReadDeviceInfoResponse {
    const COMMAND_NUMBER: u8 = READ_DEVICE_INFO;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 17 {
            return Err(DecodeError::BufferTooShort);
        }
        let pv_alarm_selection = data[0];
        let pv_transfer_function = data[1];
        let pv_unit = UnitCode::from_u8(data[2]);
        let upper_range = f32::from_be_bytes([data[3], data[4], data[5], data[6]]);
        let lower_range = f32::from_be_bytes([data[7], data[8], data[9], data[10]]);
        let damping = f32::from_be_bytes([data[11], data[12], data[13], data[14]]);
        let write_protect = data[15];
        let private_label_distributor = data[16];

        Ok(ReadDeviceInfoResponse {
            pv_alarm_selection,
            pv_transfer_function,
            pv_unit,
            upper_range,
            lower_range,
            damping,
            write_protect,
            private_label_distributor,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd15_command_number() {
        assert_eq!(ReadDeviceInfoRequest::COMMAND_NUMBER, 15);
        assert_eq!(ReadDeviceInfoResponse::COMMAND_NUMBER, 15);
    }

    #[test]
    fn test_cmd15_request_encodes_no_data() {
        let req = ReadDeviceInfoRequest;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd15_response_decode() {
        // alarm=0, transfer=0, unit=45(Meters), upper=10.0, lower=0.0, damp=1.0, wp=0, pld=0
        let upper_bytes = 10.0f32.to_be_bytes();
        let lower_bytes = 0.0f32.to_be_bytes();
        let damp_bytes = 1.0f32.to_be_bytes();
        let mut data = [0u8; 17];
        data[0] = 0x00; // alarm selection: high
        data[1] = 0x00; // transfer function: linear
        data[2] = 45; // Meters
        data[3..7].copy_from_slice(&upper_bytes);
        data[7..11].copy_from_slice(&lower_bytes);
        data[11..15].copy_from_slice(&damp_bytes);
        data[15] = 0x00; // write protect: not protected
        data[16] = 0x00; // private label distributor: none

        let resp = ReadDeviceInfoResponse::decode_data(&data).unwrap();
        assert_eq!(resp.pv_alarm_selection, 0);
        assert_eq!(resp.pv_transfer_function, 0);
        assert_eq!(resp.pv_unit, UnitCode::Meters);
        assert_eq!(resp.upper_range, 10.0f32);
        assert_eq!(resp.lower_range, 0.0f32);
        assert_eq!(resp.damping, 1.0f32);
        assert_eq!(resp.write_protect, 0);
        assert_eq!(resp.private_label_distributor, 0);
    }

    #[test]
    fn test_cmd15_response_too_short() {
        let data = [0u8; 16]; // needs 17
        assert_eq!(
            ReadDeviceInfoResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }
}
