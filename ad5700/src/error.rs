//! Error types for the AD5700 HART modem driver.

/// Low-level modem errors.
#[derive(Debug, Clone, PartialEq)]
pub enum Ad5700Error<E> {
    /// Underlying UART error.
    Uart(E),
    /// Carrier detect signal was not asserted.
    NoCarrier,
    /// Operation timed out.
    Timeout,
}

/// High-level HART communication errors.
#[derive(Debug)]
pub enum HartError<E> {
    /// Modem-level error.
    Modem(Ad5700Error<E>),
    /// Frame encoding error.
    Encode(hart_protocol::error::EncodeError),
    /// Frame decoding error.
    Decode(hart_protocol::error::DecodeError),
    /// Response timeout.
    Timeout,
}

impl<E> From<Ad5700Error<E>> for HartError<E> {
    fn from(e: Ad5700Error<E>) -> Self {
        HartError::Modem(e)
    }
}

impl<E> From<hart_protocol::error::EncodeError> for HartError<E> {
    fn from(e: hart_protocol::error::EncodeError) -> Self {
        HartError::Encode(e)
    }
}

impl<E> From<hart_protocol::error::DecodeError> for HartError<E> {
    fn from(e: hart_protocol::error::DecodeError) -> Self {
        HartError::Decode(e)
    }
}
