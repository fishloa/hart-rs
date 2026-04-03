//! Command 14 — Read Primary Variable Transducer Information

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_PV_TRANSDUCER_INFO;
use crate::error::{DecodeError, EncodeError};
use crate::units::UnitCode;

/// Command 14 request: no data payload.
#[derive(Debug, Clone)]
pub struct ReadPvTransducerInfoRequest;

/// Command 14 response: transducer serial number, limits, and minimum span.
///
/// Layout (16 bytes):
///   0..2:  `transducer_serial` (24-bit big-endian, 3 bytes)
///   3:     unit code
///   4..7:  `upper_limit` (f32 big-endian)
///   8..11: `lower_limit` (f32 big-endian)
///   12..15: `minimum_span` (f32 big-endian)
#[derive(Debug, Clone, PartialEq)]
pub struct ReadPvTransducerInfoResponse {
    /// 24-bit transducer serial number.
    pub transducer_serial: u32,
    /// Engineering unit code.
    pub unit: UnitCode,
    /// Upper transducer limit.
    pub upper_limit: f32,
    /// Lower transducer limit.
    pub lower_limit: f32,
    /// Minimum span.
    pub minimum_span: f32,
}

impl CommandRequest for ReadPvTransducerInfoRequest {
    const COMMAND_NUMBER: u8 = READ_PV_TRANSDUCER_INFO;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for ReadPvTransducerInfoResponse {
    const COMMAND_NUMBER: u8 = READ_PV_TRANSDUCER_INFO;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 16 {
            return Err(DecodeError::BufferTooShort);
        }
        let transducer_serial = super::decode_u24_be(&data[0..3]);
        let unit = UnitCode::from_u8(data[3]);
        let upper_limit = f32::from_be_bytes([data[4], data[5], data[6], data[7]]);
        let lower_limit = f32::from_be_bytes([data[8], data[9], data[10], data[11]]);
        let minimum_span = f32::from_be_bytes([data[12], data[13], data[14], data[15]]);

        Ok(ReadPvTransducerInfoResponse {
            transducer_serial,
            unit,
            upper_limit,
            lower_limit,
            minimum_span,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd14_command_number() {
        assert_eq!(ReadPvTransducerInfoRequest::COMMAND_NUMBER, 14);
        assert_eq!(ReadPvTransducerInfoResponse::COMMAND_NUMBER, 14);
    }

    #[test]
    fn test_cmd14_request_encodes_no_data() {
        let req = ReadPvTransducerInfoRequest;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd14_response_decode() {
        // serial=0x010203, unit=45(Meters), upper=10.0, lower=0.0, span=10.0
        let upper_bytes = 10.0f32.to_be_bytes();
        let lower_bytes = 0.0f32.to_be_bytes();
        let span_bytes = 10.0f32.to_be_bytes();
        let mut data = [0u8; 16];
        data[0] = 0x01;
        data[1] = 0x02;
        data[2] = 0x03;
        data[3] = 45; // Meters
        data[4..8].copy_from_slice(&upper_bytes);
        data[8..12].copy_from_slice(&lower_bytes);
        data[12..16].copy_from_slice(&span_bytes);

        let resp = ReadPvTransducerInfoResponse::decode_data(&data).unwrap();
        assert_eq!(resp.transducer_serial, 0x010203);
        assert_eq!(resp.unit, UnitCode::Meters);
        assert_eq!(resp.upper_limit, 10.0f32);
        assert_eq!(resp.lower_limit, 0.0f32);
        assert_eq!(resp.minimum_span, 10.0f32);
    }

    #[test]
    fn test_cmd14_response_too_short() {
        let data = [0u8; 15]; // needs 16
        assert_eq!(
            ReadPvTransducerInfoResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }
}
