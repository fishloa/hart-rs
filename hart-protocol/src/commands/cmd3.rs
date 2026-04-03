/// Command 3 — Read Dynamic Variables and Loop Current

use crate::consts::commands::READ_DYNAMIC_VARS;
use crate::error::{DecodeError, EncodeError};
use crate::units::UnitCode;
use super::{CommandRequest, CommandResponse};

/// Command 3 request: no data payload.
#[derive(Debug, Clone)]
pub struct Cmd3Request;

/// Command 3 response: loop current + 4 dynamic variables (PV, SV, TV, QV).
///
/// Layout (24 bytes):
///   [0..3]   loop current in mA (f32 big-endian)
///   [4]      PV unit code
///   [5..8]   PV value (f32 big-endian)
///   [9]      SV unit code
///   [10..13] SV value (f32 big-endian)
///   [14]     TV unit code
///   [15..18] TV value (f32 big-endian)
///   [19]     QV unit code
///   [20..23] QV value (f32 big-endian)
#[derive(Debug, Clone, PartialEq)]
pub struct Cmd3Response {
    pub loop_current_ma: f32,
    pub pv_unit: UnitCode,
    pub pv: f32,
    pub sv_unit: UnitCode,
    pub sv: f32,
    pub tv_unit: UnitCode,
    pub tv: f32,
    pub qv_unit: UnitCode,
    pub qv: f32,
}

impl CommandRequest for Cmd3Request {
    const COMMAND_NUMBER: u8 = READ_DYNAMIC_VARS;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for Cmd3Response {
    const COMMAND_NUMBER: u8 = READ_DYNAMIC_VARS;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 24 {
            return Err(DecodeError::BufferTooShort);
        }
        let loop_current_ma = f32::from_be_bytes([data[0], data[1], data[2], data[3]]);
        let pv_unit = UnitCode::from_u8(data[4]);
        let pv = f32::from_be_bytes([data[5], data[6], data[7], data[8]]);
        let sv_unit = UnitCode::from_u8(data[9]);
        let sv = f32::from_be_bytes([data[10], data[11], data[12], data[13]]);
        let tv_unit = UnitCode::from_u8(data[14]);
        let tv = f32::from_be_bytes([data[15], data[16], data[17], data[18]]);
        let qv_unit = UnitCode::from_u8(data[19]);
        let qv = f32::from_be_bytes([data[20], data[21], data[22], data[23]]);

        Ok(Cmd3Response {
            loop_current_ma,
            pv_unit,
            pv,
            sv_unit,
            sv,
            tv_unit,
            tv,
            qv_unit,
            qv,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd3_request_encodes_no_data() {
        let req = Cmd3Request;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd3_command_number() {
        assert_eq!(Cmd3Request::COMMAND_NUMBER, 3);
        assert_eq!(Cmd3Response::COMMAND_NUMBER, 3);
    }

    #[test]
    fn test_cmd3_response_decode_vegapuls21_style() {
        // From RESP_CMD3_LONG test vector (after stripping 2 status bytes):
        // loop_current=12.5 mA, PV=percent(57)/53.125, SV=meters(45)/2.5,
        // TV=not_used(250)/NaN, QV=celsius(32)/25.3
        let data = [
            // loop current: 12.5 mA
            0x41, 0x48, 0x00, 0x00,
            // PV: percent(57=0x39), 53.125
            0x39, 0x42, 0x54, 0x80, 0x00,
            // SV: meters(45=0x2D), 2.5
            0x2D, 0x40, 0x20, 0x00, 0x00,
            // TV: not_used(250=0xFA), NaN
            0xFA, 0x7F, 0xC0, 0x00, 0x00,
            // QV: celsius(32=0x20), 25.3
            0x20, 0x41, 0xCA, 0x66, 0x66,
        ];
        let resp = Cmd3Response::decode_data(&data).unwrap();
        assert_eq!(resp.loop_current_ma, 12.5f32);
        assert_eq!(resp.pv_unit, UnitCode::Percent);
        assert_eq!(resp.pv, 53.125f32);
        assert_eq!(resp.sv_unit, UnitCode::Meters);
        assert_eq!(resp.sv, 2.5f32);
        assert_eq!(resp.tv_unit, UnitCode::NotUsed);
        assert!(resp.tv.is_nan());
        assert_eq!(resp.qv_unit, UnitCode::DegreesCelsius);
        // 25.3 in IEEE 754 big-endian: [0x41, 0xCA, 0x66, 0x66]
        let expected_qv = f32::from_be_bytes([0x41, 0xCA, 0x66, 0x66]);
        assert_eq!(resp.qv, expected_qv);
    }

    #[test]
    fn test_cmd3_response_too_short() {
        let data = [0u8; 23]; // needs 24
        assert_eq!(
            Cmd3Response::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }
}
