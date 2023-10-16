use embedded_hal::blocking::delay::DelayMs;

pub trait IntPin: 'static {}

pub trait ResetPin: 'static {
    fn reset<D: DelayMs<u8>>(&mut self, delay: &mut D);
}
