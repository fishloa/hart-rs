use crate::consts::{delimiter, MAX_DATA_LENGTH, PREAMBLE_BYTE};
use crate::error::EncodeError;
use crate::types::{Address, FrameType};

/// Encodes a HART frame into `buf`.
///
/// Layout: [preamble x N] [delimiter] [address 1 or 5 bytes] [command] [byte_count] [data...] [checksum]
///
/// Returns the total number of bytes written, or an error if the buffer is too small
/// or the data payload exceeds MAX_DATA_LENGTH.
pub fn encode_frame(
    frame_type: FrameType,
    address: &Address,
    command: u8,
    data: &[u8],
    preamble_count: u8,
    buf: &mut [u8],
) -> Result<usize, EncodeError> {
    if data.len() > MAX_DATA_LENGTH {
        return Err(EncodeError::DataTooLong);
    }

    let addr_len = if address.is_long() { 5 } else { 1 };
    // preamble + delimiter + address + command + byte_count + data + checksum
    let total = preamble_count as usize + 1 + addr_len + 1 + 1 + data.len() + 1;

    if buf.len() < total {
        return Err(EncodeError::BufferTooSmall);
    }

    let mut pos = 0;

    // Preamble
    for _ in 0..preamble_count {
        buf[pos] = PREAMBLE_BYTE;
        pos += 1;
    }

    // Delimiter
    let delim = compute_delimiter(&frame_type, address.is_long());
    buf[pos] = delim;
    pos += 1;

    // Checksum accumulator starts at delimiter
    let mut checksum: u8 = delim;

    // Address
    let written = address.encode(&mut buf[pos..]);
    for i in 0..written {
        checksum ^= buf[pos + i];
    }
    pos += written;

    // Command
    buf[pos] = command;
    checksum ^= command;
    pos += 1;

    // Byte count
    let byte_count = data.len() as u8;
    buf[pos] = byte_count;
    checksum ^= byte_count;
    pos += 1;

    // Data
    for &b in data {
        buf[pos] = b;
        checksum ^= b;
        pos += 1;
    }

    // Checksum
    buf[pos] = checksum;
    pos += 1;

    Ok(pos)
}

fn compute_delimiter(frame_type: &FrameType, is_long: bool) -> u8 {
    match (frame_type, is_long) {
        (FrameType::Request, false) => delimiter::REQUEST_SHORT,
        (FrameType::Request, true) => delimiter::REQUEST_LONG,
        (FrameType::Response, false) => delimiter::RESPONSE_SHORT,
        (FrameType::Response, true) => delimiter::RESPONSE_LONG,
        (FrameType::Burst, false) => delimiter::BURST_SHORT,
        (FrameType::Burst, true) => delimiter::BURST_LONG,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consts::MIN_PREAMBLE_COUNT;
    use crate::types::{Address, FrameType, MasterRole};

    // Test vector from test_vectors.rs: REQ_CMD0_SHORT_PRIMARY
    const REQ_CMD0_SHORT_PRIMARY: &[u8] = &[
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // preamble (5)
        0x02, // delimiter: request short
        0x80, // address: primary master, poll addr 0
        0x00, // command 0
        0x00, // byte count 0
        0x82, // checksum
    ];

    // Test vector from test_vectors.rs: REQ_CMD0_LONG_PRIMARY
    const REQ_CMD0_LONG_PRIMARY: &[u8] = &[
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0x82, // delimiter: request long
        0x9A, 0x2B, 0x11, 0x22, 0x33, // long address
        0x00, // command 0
        0x00, // byte count 0
        0x33, // checksum
    ];

    #[test]
    fn test_encode_short_request_no_data_matches_test_vector() {
        let address = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; 32];
        let len = encode_frame(
            FrameType::Request,
            &address,
            0,
            &[],
            MIN_PREAMBLE_COUNT,
            &mut buf,
        )
        .unwrap();

        assert_eq!(&buf[..len], REQ_CMD0_SHORT_PRIMARY);
        // Verify preambles
        assert!(buf[..5].iter().all(|&b| b == 0xFF));
        // Verify delimiter
        assert_eq!(buf[5], 0x02);
        // Verify address
        assert_eq!(buf[6], 0x80);
        // Verify command
        assert_eq!(buf[7], 0x00);
        // Verify checksum
        assert_eq!(buf[9], 0x82);
    }

    #[test]
    fn test_encode_long_request_with_data_matches_test_vector() {
        let address = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x1A,
            device_type: 0x2B,
            device_id: 0x112233,
        };
        let mut buf = [0u8; 32];
        let len = encode_frame(
            FrameType::Request,
            &address,
            0,
            &[],
            MIN_PREAMBLE_COUNT,
            &mut buf,
        )
        .unwrap();

        assert_eq!(&buf[..len], REQ_CMD0_LONG_PRIMARY);
    }

    #[test]
    fn test_encode_long_request_with_payload() {
        let address = Address::Long {
            master: MasterRole::Primary,
            burst: false,
            manufacturer_id: 0x1A,
            device_type: 0x2B,
            device_id: 0x112233,
        };
        let data = [0xAB, 0xCD];
        let mut buf = [0u8; 64];
        let len = encode_frame(
            FrameType::Request,
            &address,
            0x10,
            &data,
            MIN_PREAMBLE_COUNT,
            &mut buf,
        )
        .unwrap();
        // preamble(5) + delimiter(1) + address(5) + cmd(1) + byte_count(1) + data(2) + checksum(1) = 16
        assert_eq!(len, 16);
        // Check delimiter is request long
        assert_eq!(buf[5], 0x82);
        // Verify byte_count
        assert_eq!(buf[12], 0x02);
        // Verify data bytes
        assert_eq!(buf[13], 0xAB);
        assert_eq!(buf[14], 0xCD);
        // Verify checksum
        let expected_cs = buf[5..len - 1].iter().fold(0u8, |acc, &b| acc ^ b);
        assert_eq!(buf[len - 1], expected_cs);
    }

    #[test]
    fn test_encode_buffer_too_small() {
        let address = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; 4]; // too small
        let result = encode_frame(FrameType::Request, &address, 0, &[], MIN_PREAMBLE_COUNT, &mut buf);
        assert_eq!(result, Err(EncodeError::BufferTooSmall));
    }

    #[test]
    fn test_encode_data_too_long() {
        let address = Address::Short {
            master: MasterRole::Primary,
            burst: false,
            poll_address: 0,
        };
        let data = [0u8; 256]; // > MAX_DATA_LENGTH (255)
        let mut buf = [0u8; 512];
        let result = encode_frame(FrameType::Request, &address, 0, &data, MIN_PREAMBLE_COUNT, &mut buf);
        assert_eq!(result, Err(EncodeError::DataTooLong));
    }

    #[test]
    fn test_encode_secondary_master_short() {
        // REQ_CMD0_SHORT_SECONDARY from test_vectors.rs
        const REQ_CMD0_SHORT_SECONDARY: &[u8] = &[
            0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
            0x02,
            0x00, // secondary master, poll addr 0
            0x00,
            0x00,
            0x02,
        ];
        let address = Address::Short {
            master: MasterRole::Secondary,
            burst: false,
            poll_address: 0,
        };
        let mut buf = [0u8; 32];
        let len = encode_frame(
            FrameType::Request,
            &address,
            0,
            &[],
            MIN_PREAMBLE_COUNT,
            &mut buf,
        )
        .unwrap();
        assert_eq!(&buf[..len], REQ_CMD0_SHORT_SECONDARY);
    }
}
