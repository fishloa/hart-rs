//! Command 8 — Read Dynamic Variable Classifications

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_DYNAMIC_VAR_CLASS;
use crate::error::{DecodeError, EncodeError};

/// Command 8 request: no data payload.
#[derive(Debug, Clone)]
pub struct ReadDynamicVarClassRequest;

/// Command 8 response: classifications for PV, SV, TV, QV.
///
/// Layout (4 bytes):
///   [0] pv_classification
///   [1] sv_classification
///   [2] tv_classification
///   [3] qv_classification
#[derive(Debug, Clone, PartialEq)]
pub struct ReadDynamicVarClassResponse {
    pub pv_classification: u8,
    pub sv_classification: u8,
    pub tv_classification: u8,
    pub qv_classification: u8,
}

impl CommandRequest for ReadDynamicVarClassRequest {
    const COMMAND_NUMBER: u8 = READ_DYNAMIC_VAR_CLASS;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for ReadDynamicVarClassResponse {
    const COMMAND_NUMBER: u8 = READ_DYNAMIC_VAR_CLASS;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 4 {
            return Err(DecodeError::BufferTooShort);
        }
        Ok(ReadDynamicVarClassResponse {
            pv_classification: data[0],
            sv_classification: data[1],
            tv_classification: data[2],
            qv_classification: data[3],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd8_request_encodes_no_data() {
        let req = ReadDynamicVarClassRequest;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd8_response_decode() {
        // classification codes: 0=not_classified, 1=temperature, 2=pressure, etc.
        let data = [0x01u8, 0x02, 0x00, 0x00];
        let resp = ReadDynamicVarClassResponse::decode_data(&data).unwrap();
        assert_eq!(resp.pv_classification, 1);
        assert_eq!(resp.sv_classification, 2);
        assert_eq!(resp.tv_classification, 0);
        assert_eq!(resp.qv_classification, 0);
    }

    #[test]
    fn test_cmd8_response_too_short() {
        let data = [0x01u8, 0x02, 0x00]; // needs 4
        assert_eq!(
            ReadDynamicVarClassResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd8_command_number() {
        assert_eq!(ReadDynamicVarClassRequest::COMMAND_NUMBER, 8);
        assert_eq!(ReadDynamicVarClassResponse::COMMAND_NUMBER, 8);
    }
}
