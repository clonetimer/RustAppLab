//! Explicit in-memory HAL test doubles. These are NOT board GPIO or ADC drivers.
use crate::Sensor;
use embedded_hal::digital::{ErrorKind, ErrorType, InputPin, OutputPin};

pub struct SimSensor {
    pub raw: u16,
    pub fail: bool,
}
impl Sensor for SimSensor {
    type Error = ();
    fn read(&mut self) -> Result<u16, Self::Error> {
        if self.fail { Err(()) } else { Ok(self.raw) }
    }
}

pub struct SimInput {
    pub high: bool,
    pub fail: bool,
}
impl ErrorType for SimInput { type Error = ErrorKind; }
impl InputPin for SimInput {
    fn is_high(&mut self) -> Result<bool, Self::Error> {
        if self.fail { Err(ErrorKind::Other) } else { Ok(self.high) }
    }
    fn is_low(&mut self) -> Result<bool, Self::Error> {
        self.is_high().map(|high| !high)
    }
}

pub struct SimOutput {
    pub high: bool,
    pub fail_high: bool,
    pub fail_low: bool,
}
impl SimOutput {
    pub const fn new() -> Self { Self { high: false, fail_high: false, fail_low: false } }
}
impl Default for SimOutput {
    fn default() -> Self { Self::new() }
}
impl ErrorType for SimOutput { type Error = ErrorKind; }
impl OutputPin for SimOutput {
    fn set_high(&mut self) -> Result<(), Self::Error> {
        if self.fail_high { Err(ErrorKind::Other) } else { self.high = true; Ok(()) }
    }
    fn set_low(&mut self) -> Result<(), Self::Error> {
        if self.fail_low { Err(ErrorKind::Other) } else { self.high = false; Ok(()) }
    }
}
