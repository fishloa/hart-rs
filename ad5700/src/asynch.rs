//! Async AD5700 HART modem driver.

use embedded_hal::digital::{InputPin, OutputPin};
use embedded_io_async::{ErrorType, Read, Write};

use crate::error::Ad5700Error;

/// Async AD5700-1 HART modem driver.
///
/// `UART` must implement [`embedded_io_async::Read`] and [`embedded_io_async::Write`].
/// `RTS` must implement [`embedded_hal::digital::OutputPin`] (request-to-send, sync).
/// `CD` must implement [`embedded_hal::digital::InputPin`] (carrier detect, sync).
pub struct Ad5700Async<UART, RTS, CD> {
    uart: UART,
    rts: RTS,
    cd: CD,
}

impl<UART, RTS, CD> Ad5700Async<UART, RTS, CD>
where
    UART: Read + Write,
    RTS: OutputPin,
    CD: InputPin,
{
    /// Create a new async AD5700 driver from its constituent peripherals.
    pub fn new(uart: UART, rts: RTS, cd: CD) -> Self {
        Ad5700Async { uart, rts, cd }
    }

    /// Transmit `data` over the HART bus asynchronously.
    ///
    /// Asserts RTS before writing, then deasserts RTS after flushing.
    ///
    /// # Errors
    ///
    /// Returns [`Ad5700Error::Uart`] on UART write/flush failures or
    /// [`Ad5700Error::NoCarrier`] if RTS toggling fails.
    pub async fn transmit(
        &mut self,
        data: &[u8],
    ) -> Result<(), Ad5700Error<<UART as ErrorType>::Error>> {
        self.rts.set_high().map_err(|_| Ad5700Error::NoCarrier)?;
        self.uart.write_all(data).await.map_err(Ad5700Error::Uart)?;
        self.uart.flush().await.map_err(Ad5700Error::Uart)?;
        self.rts.set_low().map_err(|_| Ad5700Error::NoCarrier)?;
        Ok(())
    }

    /// Read available bytes into `buf` asynchronously.
    ///
    /// Returns the number of bytes read.
    ///
    /// # Errors
    ///
    /// Returns [`Ad5700Error::Uart`] on UART read failures.
    pub async fn receive_into(
        &mut self,
        buf: &mut [u8],
    ) -> Result<usize, Ad5700Error<<UART as ErrorType>::Error>> {
        let n = self.uart.read(buf).await.map_err(Ad5700Error::Uart)?;
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
