//! Error types for HART frame encoding and decoding.

/// Errors that can occur while encoding a HART frame.
#[derive(Debug, Clone, PartialEq)]
pub enum EncodeError {
    /// The output buffer is too small for the encoded frame.
    BufferTooSmall,
    /// The data payload exceeds the maximum allowed length.
    DataTooLong,
}

/// Errors that can occur while decoding a HART frame.
#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    /// The input buffer does not contain enough bytes.
    BufferTooShort,
    /// The delimiter byte does not match any known frame type.
    InvalidDelimiter(
        /// The unrecognised delimiter byte value.
        u8,
    ),
    /// The computed checksum does not match the received checksum.
    ChecksumMismatch {
        /// The checksum value computed from the frame bytes.
        expected: u8,
        /// The checksum byte actually received.
        actual: u8,
    },
}
