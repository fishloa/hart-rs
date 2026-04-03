//! Async HART master for Embassy, built on the AD5700 async modem.

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{ErrorType, Read, Write};
use embassy_time::{Duration, with_timeout};

use hart_protocol::{
    commands::{CommandRequest, CommandResponse},
    consts::{DEFAULT_PREAMBLE_COUNT, MAX_FRAME_LENGTH, RESPONSE_TIMEOUT_MS},
    decode::Decoder,
    encode::encode_frame,
    types::{Address, FrameType, ResponseStatus},
};

use ad5700::{
    asynch::Ad5700Async,
    error::HartError,
};

/// Async HART master controller for Embassy.
///
/// Wraps an [`Ad5700Async`] modem, a frame [`Decoder`], and internal transmit/receive
/// buffers. Provides a single `send_command` async method that handles the full
/// request–response cycle with a configurable timeout.
pub struct HartMaster<UART, RTS, CD> {
    modem: Ad5700Async<UART, RTS, CD>,
    decoder: Decoder,
    tx_buf: [u8; MAX_FRAME_LENGTH],
    rx_buf: [u8; MAX_FRAME_LENGTH],
    preamble_count: u8,
}

impl<UART, RTS, CD> HartMaster<UART, RTS, CD>
where
    UART: Read + Write,
    RTS: OutputPin,
    CD: InputPin,
{
    /// Create a new async HART master wrapping the given modem.
    pub fn new(modem: Ad5700Async<UART, RTS, CD>) -> Self {
        HartMaster {
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

    /// Send a HART command request and receive the response asynchronously.
    ///
    /// The receive phase is wrapped in a [`RESPONSE_TIMEOUT_MS`]-millisecond timeout.
    /// Returns `HartError::Timeout` if no complete frame arrives in time.
    pub async fn send_command<Req, Resp>(
        &mut self,
        address: &Address,
        request: &Req,
    ) -> Result<(ResponseStatus, Resp), HartError<<UART as ErrorType>::Error>>
    where
        Req: CommandRequest,
        Resp: CommandResponse,
    {
        // Encode request payload
        let mut data_buf = [0u8; 255];
        let data_len = request
            .encode_data(&mut data_buf)
            .map_err(HartError::Encode)?;

        // Encode the full HART frame
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
            .await
            .map_err(HartError::Modem)?;

        // Receive and decode the response, with timeout
        self.decoder.reset();

        let receive_fut = self.receive_response::<Resp>();
        with_timeout(
            Duration::from_millis(RESPONSE_TIMEOUT_MS as u64),
            receive_fut,
        )
        .await
        .map_err(|_| HartError::Timeout)?
    }

    /// Inner future: read bytes from the modem until a complete HART frame is decoded.
    async fn receive_response<Resp: CommandResponse>(
        &mut self,
    ) -> Result<(ResponseStatus, Resp), HartError<<UART as ErrorType>::Error>> {
        loop {
            let n = self
                .modem
                .receive_into(&mut self.rx_buf)
                .await
                .map_err(HartError::Modem)?;

            if n == 0 {
                return Err(HartError::Timeout);
            }

            for i in 0..n {
                match self.decoder.feed(self.rx_buf[i]) {
                    Ok(None) => {}
                    Ok(Some(frame)) => {
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
