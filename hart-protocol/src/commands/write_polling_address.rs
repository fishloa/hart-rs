//! Command 6 — Write Polling Address

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::WRITE_POLLING_ADDRESS;
use crate::error::{DecodeError, EncodeError};

/// Command 6 request: polling address and loop current mode.
///
/// Layout (2 bytes):
///   [0] polling_address
///   [1] loop_current_mode
#[derive(Debug, Clone, PartialEq)]
pub struct WritePollingAddressRequest {
    pub polling_address: u8,
    pub loop_current_mode: u8,
}

/// Command 6 response: echoes polling address and loop current mode.
///
/// Layout (2 bytes):
///   [0] polling_address
///   [1] loop_current_mode
#[derive(Debug, Clone, PartialEq)]
pub struct WritePollingAddressResponse {
    pub polling_address: u8,
    pub loop_current_mode: u8,
}

impl CommandRequest for WritePollingAddressRequest {
    const COMMAND_NUMBER: u8 = WRITE_POLLING_ADDRESS;

    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError> {
        if buf.len() < 2 {
            return Err(EncodeError::BufferTooSmall);
        }
        buf[0] = self.polling_address;
        buf[1] = self.loop_current_mode;
        Ok(2)
    }
}

impl CommandResponse for WritePollingAddressResponse {
    const COMMAND_NUMBER: u8 = WRITE_POLLING_ADDRESS;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 2 {
            return Err(DecodeError::BufferTooShort);
        }
        Ok(WritePollingAddressResponse {
            polling_address: data[0],
            loop_current_mode: data[1],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmd6_command_number() {
        assert_eq!(WritePollingAddressRequest::COMMAND_NUMBER, 6);
        assert_eq!(WritePollingAddressResponse::COMMAND_NUMBER, 6);
    }

    #[test]
    fn test_cmd6_request_encode() {
        let req = WritePollingAddressRequest {
            polling_address: 3,
            loop_current_mode: 1,
        };
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 2);
        assert_eq!(buf[0], 3);
        assert_eq!(buf[1], 1);
    }

    #[test]
    fn test_cmd6_request_buffer_too_small() {
        let req = WritePollingAddressRequest {
            polling_address: 0,
            loop_current_mode: 0,
        };
        let mut buf = [0u8; 1]; // too small
        assert_eq!(req.encode_data(&mut buf), Err(EncodeError::BufferTooSmall));
    }

    #[test]
    fn test_cmd6_response_decode() {
        let data = [0x03u8, 0x01]; // polling_address=3, loop_current_mode=1
        let resp = WritePollingAddressResponse::decode_data(&data).unwrap();
        assert_eq!(resp.polling_address, 3);
        assert_eq!(resp.loop_current_mode, 1);
    }

    #[test]
    fn test_cmd6_response_too_short() {
        let data = [0x03u8]; // needs 2
        assert_eq!(
            WritePollingAddressResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }

    #[test]
    fn test_cmd6_roundtrip() {
        let req = WritePollingAddressRequest {
            polling_address: 5,
            loop_current_mode: 0,
        };
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        let resp = WritePollingAddressResponse::decode_data(&buf[..len]).unwrap();
        assert_eq!(resp.polling_address, req.polling_address);
        assert_eq!(resp.loop_current_mode, req.loop_current_mode);
    }
}
