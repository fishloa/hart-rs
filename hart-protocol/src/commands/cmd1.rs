/// Command 1 — Read Primary Variable

use crate::consts::commands::READ_PRIMARY_VARIABLE;
use crate::error::{DecodeError, EncodeError};
use crate::units::UnitCode;
use super::{CommandRequest, CommandResponse};

/// Command 1 request: no data payload.
#[derive(Debug, Clone)]
pub struct Cmd1Request;

/// Command 1 response: primary variable unit and value.
///
/// Layout (5 bytes):
///   [0]    unit code
///   [1..4] PV value (f32 big-endian)
#[derive(Debug, Clone, PartialEq)]
pub struct Cmd1Response {
    pub unit: UnitCode,
    pub value: f32,
}

impl CommandRequest for Cmd1Request {
    const COMMAND_NUMBER: u8 = READ_PRIMARY_VARIABLE;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for Cmd1Response {
    const COMMAND_NUMBER: u8 = READ_PRIMARY_VARIABLE;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 5 {
            return Err(DecodeError::BufferTooShort);
        }
        let unit = UnitCode::from_u8(data[0]);
        let value = f32::from_be_bytes([data[1], data[2], data[3], data[4]]);
        Ok(Cmd1Response { unit, value })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd1_request_encodes_no_data() {
        let req = Cmd1Request;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd1_response_decode_meters_3_14() {
        // From RESP_CMD1_LONG test vector (after stripping 2 status bytes):
        // unit=0x2D (45=meters), value=3.14
        let data = [0x2D, 0x40, 0x48, 0xF5, 0xC3];
        let resp = Cmd1Response::decode_data(&data).unwrap();
        assert_eq!(resp.unit, UnitCode::Meters);
        // 3.14 in IEEE 754 round-trips exactly here
        let expected = f32::from_be_bytes([0x40, 0x48, 0xF5, 0xC3]);
        assert_eq!(resp.value, expected);
    }

    #[test]
    fn test_cmd1_response_too_short() {
        let data = [0x2D, 0x40, 0x48]; // only 3 bytes
        assert_eq!(
            Cmd1Response::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd1_command_number() {
        assert_eq!(Cmd1Request::COMMAND_NUMBER, 1);
        assert_eq!(Cmd1Response::COMMAND_NUMBER, 1);
    }
}
