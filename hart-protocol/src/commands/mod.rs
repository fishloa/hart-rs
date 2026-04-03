use crate::error::{DecodeError, EncodeError};

pub mod cmd0;
pub mod cmd1;
pub mod cmd2;
pub mod cmd3;
pub mod cmd6;
pub mod cmd7;
pub mod cmd8;
pub mod cmd9;
pub mod cmd11;
pub mod cmd12;
pub mod cmd13;
pub mod cmd14;
pub mod cmd15;
pub mod cmd16;
pub mod cmd17;
pub mod cmd18;
pub mod cmd19;
pub mod cmd20;
pub mod cmd22;
pub mod cmd38;
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
