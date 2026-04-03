//! Blocking HART master built on the AD5700 modem.

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io::{ErrorType, Read, Write};

use hart_protocol::{
    commands::{CommandRequest, CommandResponse},
    consts::{DEFAULT_PREAMBLE_COUNT, MAX_FRAME_LENGTH},
    decode::Decoder,
    encode::encode_frame,
    types::{Address, FrameType, ResponseStatus},
};

use crate::{
    blocking::Ad5700,
    error::HartError,
};

/// Blocking HART master controller.
///
/// Wraps an [`Ad5700`] modem, a frame [`Decoder`], and internal transmit/receive
/// buffers. Provides a single `send_command` method that handles the full
/// request–response cycle.
pub struct HartMasterBlocking<UART, RTS, CD> {
    modem: Ad5700<UART, RTS, CD>,
    decoder: Decoder,
    tx_buf: [u8; MAX_FRAME_LENGTH],
    rx_buf: [u8; MAX_FRAME_LENGTH],
    preamble_count: u8,
}

impl<UART, RTS, CD> HartMasterBlocking<UART, RTS, CD>
where
    UART: Read + Write,
    RTS: OutputPin,
    CD: InputPin,
{
    /// Create a new blocking HART master wrapping the given modem.
    pub fn new(modem: Ad5700<UART, RTS, CD>) -> Self {
        HartMasterBlocking {
            modem,
            decoder: Decoder::new(),
            tx_buf: [0u8; MAX_FRAME_LENGTH],
            rx_buf: [0u8; MAX_FRAME_LENGTH],
            preamble_count: DEFAULT_PREAMBLE_COUNT,
        }
    }

    /// Override the preamble byte count used for outgoing frames (default 10).
    pub fn set_preamble_count(&mut self, count: u8) {
        self.preamble_count = count;
    }

    /// Send a HART command request and receive the response.
    ///
    /// Encodes the request, transmits it, receives bytes, feeds them into the
    /// decoder, then parses and returns the `(ResponseStatus, Resp)` pair.
    pub fn send_command<Req, Resp>(
        &mut self,
        address: &Address,
        request: &Req,
    ) -> Result<(ResponseStatus, Resp), HartError<<UART as ErrorType>::Error>>
    where
        Req: CommandRequest,
        Resp: CommandResponse,
    {
        // Encode request payload into a temporary buffer
        let mut data_buf = [0u8; 255];
        let data_len = request
            .encode_data(&mut data_buf)
            .map_err(HartError::Encode)?;

        // Encode the full HART frame into tx_buf
        let frame_len = encode_frame(
            FrameType::Request,
            address,
            Req::COMMAND_NUMBER,
            &data_buf[..data_len],
            self.preamble_count,
            &mut self.tx_buf,
        )
        .map_err(HartError::Encode)?;

        // Transmit the frame
        self.modem
            .transmit(&self.tx_buf[..frame_len])
            .map_err(HartError::Modem)?;

        // Receive and decode the response frame
        self.decoder.reset();
        loop {
            let n = self
                .modem
                .receive_into(&mut self.rx_buf)
                .map_err(HartError::Modem)?;

            if n == 0 {
                return Err(HartError::Timeout);
            }

            for i in 0..n {
                match self.decoder.feed(self.rx_buf[i]) {
                    Ok(None) => {}
                    Ok(Some(frame)) => {
                        // Response data: first 2 bytes are status, rest is payload
                        let data = frame.data.as_slice();
                        if data.len() < 2 {
                            return Err(HartError::Decode(
                                hart_protocol::error::DecodeError::BufferTooShort,
                            ));
                        }
                        let status = ResponseStatus::from_bytes([data[0], data[1]]);
                        let resp = Resp::decode_data(&data[2..]).map_err(HartError::Decode)?;
                        return Ok((status, resp));
                    }
                    Err(e) => return Err(HartError::Decode(e)),
                }
            }
        }
    }
}
