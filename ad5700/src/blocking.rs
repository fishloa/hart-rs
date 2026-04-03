//! Blocking AD5700 HART modem driver.

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io::{ErrorType, Read, Write};

use crate::error::Ad5700Error;

/// Blocking AD5700-1 HART modem driver.
///
/// `UART` must implement [`embedded_io::Read`] and [`embedded_io::Write`].
/// `RTS` must implement [`embedded_hal::digital::OutputPin`] (request-to-send).
/// `CD` must implement [`embedded_hal::digital::InputPin`] (carrier detect).
pub struct Ad5700<UART, RTS, CD> {
    uart: UART,
    rts: RTS,
    cd: CD,
}

impl<UART, RTS, CD> Ad5700<UART, RTS, CD>
where
    UART: Read + Write,
    RTS: OutputPin,
    CD: InputPin,
{
    /// Create a new AD5700 driver from its constituent peripherals.
    pub fn new(uart: UART, rts: RTS, cd: CD) -> Self {
        Ad5700 { uart, rts, cd }
    }

    /// Transmit `data` over the HART bus.
    ///
    /// Asserts RTS before writing, then deasserts RTS after flushing.
    pub fn transmit(&mut self, data: &[u8]) -> Result<(), Ad5700Error<<UART as ErrorType>::Error>> {
        self.rts.set_high().map_err(|_| Ad5700Error::NoCarrier)?;
        self.uart.write_all(data).map_err(Ad5700Error::Uart)?;
        self.uart.flush().map_err(Ad5700Error::Uart)?;
        self.rts.set_low().map_err(|_| Ad5700Error::NoCarrier)?;
        Ok(())
    }

    /// Read available bytes into `buf`.
    ///
    /// Returns the number of bytes read. Stops when no more bytes are immediately
    /// available (i.e. a single `read` call returns 0 or fills `buf`).
    pub fn receive_into(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, Ad5700Error<<UART as ErrorType>::Error>> {
        let n = self.uart.read(buf).map_err(Ad5700Error::Uart)?;
        Ok(n)
    }

    /// Returns `true` if a HART carrier signal is currently detected.
    pub fn carrier_detected(&mut self) -> bool {
        self.cd.is_high().unwrap_or(false)
    }

    /// Release the underlying peripherals, consuming the driver.
    pub fn release(self) -> (UART, RTS, CD) {
        (self.uart, self.rts, self.cd)
    }
}
