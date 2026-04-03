#[derive(Debug, Clone, PartialEq)]
pub enum EncodeError {
    BufferTooSmall,
    DataTooLong,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DecodeError {
    BufferTooShort,
    InvalidDelimiter(u8),
    ChecksumMismatch { expected: u8, actual: u8 },
    InvalidAddress,
    InvalidFrameType,
}
