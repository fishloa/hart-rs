//! Command 13 — Read Tag, Descriptor, and Date

use super::{CommandRequest, CommandResponse};
use crate::consts::commands::READ_TAG_DESCRIPTOR_DATE;
use crate::error::{DecodeError, EncodeError};
use crate::packed_string::decode_packed;

/// Command 13 request: no data payload.
#[derive(Debug, Clone)]
pub struct ReadTagDescriptorDateRequest;

/// Command 13 response: tag (8 chars), descriptor (16 chars), and date (day/month/year).
///
/// Layout (21 bytes):
///   [0..5]   tag: 6 packed bytes → 8 ASCII chars
///   [6..17]  descriptor: 12 packed bytes → 16 ASCII chars
///   [18]     day
///   [19]     month
///   [20]     year (years since 1900)
#[derive(Debug, Clone, PartialEq)]
pub struct ReadTagDescriptorDateResponse {
    pub tag: [u8; 8],
    pub descriptor: [u8; 16],
    pub day: u8,
    pub month: u8,
    pub year: u8,
}

impl CommandRequest for ReadTagDescriptorDateRequest {
    const COMMAND_NUMBER: u8 = READ_TAG_DESCRIPTOR_DATE;

    fn encode_data(&self, _buf: &mut [u8]) -> Result<usize, EncodeError> {
        Ok(0)
    }
}

impl CommandResponse for ReadTagDescriptorDateResponse {
    const COMMAND_NUMBER: u8 = READ_TAG_DESCRIPTOR_DATE;

    fn decode_data(data: &[u8]) -> Result<Self, DecodeError> {
        if data.len() < 21 {
            return Err(DecodeError::BufferTooShort);
        }
        let mut tag = [b' '; 8];
        decode_packed(&data[0..6], &mut tag);

        let mut descriptor = [b' '; 16];
        decode_packed(&data[6..18], &mut descriptor);

        let day = data[18];
        let month = data[19];
        let year = data[20];

        Ok(ReadTagDescriptorDateResponse {
            tag,
            descriptor,
            day,
            month,
            year,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::packed_string::encode_packed;

    #[test]
    fn test_cmd13_command_number() {
        assert_eq!(ReadTagDescriptorDateRequest::COMMAND_NUMBER, 13);
        assert_eq!(ReadTagDescriptorDateResponse::COMMAND_NUMBER, 13);
    }

    #[test]
    fn test_cmd13_request_encodes_no_data() {
        let req = ReadTagDescriptorDateRequest;
        let mut buf = [0u8; 4];
        let len = req.encode_data(&mut buf).unwrap();
        assert_eq!(len, 0);
    }

    #[test]
    fn test_cmd13_response_decode() {
        let tag_str = b"SENSOR01";
        let desc_str = b"MAIN SENSOR DESC";
        let mut data = [0u8; 21];

        // Encode tag (8 chars → 6 bytes)
        encode_packed(tag_str, &mut data[0..6]);
        // Encode descriptor (16 chars → 12 bytes)
        encode_packed(desc_str, &mut data[6..18]);
        // Date
        data[18] = 15; // day
        data[19] = 6; // month
        data[20] = 123; // year (1900+123 = 2023)

        let resp = ReadTagDescriptorDateResponse::decode_data(&data).unwrap();
        assert_eq!(&resp.tag, tag_str);
        assert_eq!(&resp.descriptor, desc_str);
        assert_eq!(resp.day, 15);
        assert_eq!(resp.month, 6);
        assert_eq!(resp.year, 123);
    }

    #[test]
    fn test_cmd13_response_too_short() {
        let data = [0u8; 20]; // needs 21
        assert_eq!(
            ReadTagDescriptorDateResponse::decode_data(&data),
            Err(DecodeError::BufferTooShort)
        );
    }
}
