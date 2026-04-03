use crate::error::{DecodeError, EncodeError};

pub mod cmd0;
pub mod cmd1;
pub mod cmd2;
pub mod cmd3;
pub mod cmd48;

/// Trait for HART command requests.
pub trait CommandRequest {
    const COMMAND_NUMBER: u8;
    fn encode_data(&self, buf: &mut [u8]) -> Result<usize, EncodeError>;
}

/// Trait for HART command responses.
pub trait CommandResponse: Sized {
    const COMMAND_NUMBER: u8;
    fn decode_data(data: &[u8]) -> Result<Self, DecodeError>;
}
