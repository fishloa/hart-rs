//! Command 7 — Read Loop Configuration

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_LOOP_CONFIG;
use crate::error::{DecodeError, EncodeError};

/// Command 7 request: no data payload.
#[derive(Debug, Clone)]
pub struct ReadLoopConfigRequest;

/// Command 7 response: loop polling address and current mode.
///
/// Layout (2 bytes):
///   [0] polling_address
///   [1] loop_current_mode
#[derive(Debug, Clone, PartialEq)]
pub struct ReadLoopConfigResponse {
    pub polling_address: u8,
    pub loop_current_mode: u8,
}

impl CommandRequest for ReadLoopConfigRequest {
    const COMMAND_NUMBER: u8 = READ_LOOP_CONFIG;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for ReadLoopConfigResponse {
    const COMMAND_NUMBER: u8 = READ_LOOP_CONFIG;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 2 {
            return Err(DecodeError::BufferTooShort);
        }
        Ok(ReadLoopConfigResponse {
            polling_address: data[0],
            loop_current_mode: data[1],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd7_request_encodes_no_data() {
        let req = ReadLoopConfigRequest;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
        assert_eq!(ReadLoopConfigRequest::COMMAND_NUMBER, 7);
    }

    #[test]
    fn test_cmd7_response_decode() {
        let data = [0x00u8, 0x01]; // polling_address=0, loop_current_mode=1 (enabled)
        let resp = ReadLoopConfigResponse::decode_data(&data).unwrap();
        assert_eq!(resp.polling_address, 0);
        assert_eq!(resp.loop_current_mode, 1);
    }

    #[test]
    fn test_cmd7_response_too_short() {
        let data = [0x00u8]; // needs 2
        assert_eq!(
            ReadLoopConfigResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd7_command_number() {
        assert_eq!(ReadLoopConfigRequest::COMMAND_NUMBER, 7);
        assert_eq!(ReadLoopConfigResponse::COMMAND_NUMBER, 7);
    }
}
