//! todo

use crate::Enc28j60;
use embedded_hal::{blocking, digital::OutputPin};
use smoltcp::{
    phy::{self, Device, DeviceCapabilities},
    time::Instant,
};

/// Wrapper for Enc28j60 for use as a `smoltcp` interface
pub struct Phy<'a, SPI, NCS, INT, RESET> {
    phy: Enc28j60<SPI, NCS, INT, RESET>,
    rx_buf: &'a mut [u8],
}

impl<'a, SPI, NCS, INT, RESET> Phy<'a, SPI, NCS, INT, RESET> {
    /// Create a new Eth from an Enc28j60 and a receive buffer
    pub fn new(phy: Enc28j60<SPI, NCS, INT, RESET>, rx_buf: &'a mut [u8]) -> Self {
        Phy { phy, rx_buf }
    }
}

impl<'a, 'b, E, SPI: 'a, NCS: 'a, INT, RESET> Device<'a> for &mut Phy<'b, SPI, NCS, INT, RESET>
where
    SPI: blocking::spi::Transfer<u8, Error = E> + blocking::spi::Write<u8, Error = E>,
    NCS: OutputPin,
    INT: crate::sealed::IntPin,
    RESET: crate::sealed::ResetPin,
{
    type RxToken = RxToken<'a>;
    type TxToken = TxToken<'a, SPI, NCS, INT, RESET>;

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        let packet = self.phy.next_packet().ok().unwrap();
        match packet {
            Some(packet) => {
                packet.read(&mut self.rx_buf[..]).ok().unwrap();
                Some((
                    RxToken(&mut self.rx_buf[..]),
                    TxToken {
                        phy: &mut self.phy,
                        buf: [0u8; 1024],
                    },
                ))
            }
            None => None,
        }
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(TxToken {
            phy: &mut self.phy,
            buf: [0u8; 1024],
        })
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1500;
        caps
    }
}

/// ?
pub struct RxToken<'a>(&'a [u8]);

impl<'a> phy::RxToken for RxToken<'a> {
    fn consume<R, F>(self, _timestamp: Instant, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&[u8]) -> smoltcp::Result<R>,
    {
        let result = f(self.0);
        result
    }
}

/// ?
pub struct TxToken<'a, SPI, NCS, INT, RESET> {
    phy: &'a mut Enc28j60<SPI, NCS, INT, RESET>,
    buf: [u8; 1024],
}

impl<'a, E, SPI, NCS, INT, RESET> phy::TxToken for TxToken<'a, SPI, NCS, INT, RESET>
where
    SPI: blocking::spi::Transfer<u8, Error = E> + blocking::spi::Write<u8, Error = E>,
    NCS: OutputPin,
    INT: crate::sealed::IntPin,
    RESET: crate::sealed::ResetPin,
{
    fn consume<R, F>(mut self, _timestamp: Instant, len: usize, f: F) -> smoltcp::Result<R>
    where
        F: FnOnce(&mut [u8]) -> smoltcp::Result<R>,
    {
        let result = f(&mut self.buf[..len]);
        self.phy.transmit(&self.buf[..len]).ok().unwrap();
        result
    }
}