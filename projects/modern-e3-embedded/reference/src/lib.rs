#![no_std]
//! Shared embedded control logic: no allocator, operating system or clock I/O.
//! QEMU exercises actual CPU/interrupt execution; simulated pins are NOT physical IO.
use embedded_hal::digital::{InputPin, OutputPin};

pub const RAW_MAX: u16 = 4095;
const MAX_INTERVAL: u32 = i32::MAX as u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    InvalidConfig,
    ClockAmbiguous,
    Sensor,
    Interlock,
    Output,
    Latched,
    Todo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Record {
    pub scheduled: u32,
    pub raw: u16,
    pub enabled: bool,
    pub missed: u32,
}

/// Fixed-capacity history. Its logical index 0 is the oldest retained record.
/// A zero-capacity buffer rejects insertion instead of dividing by zero.
pub struct History<const N: usize> {
    slots: [Option<Record>; N],
    oldest: usize,
    len: usize,
}

impl<const N: usize> Default for History<N> {
    fn default() -> Self { Self::new() }
}

impl<const N: usize> History<N> {
    pub const fn new() -> Self {
        Self { slots: [None; N], oldest: 0, len: 0 }
    }

    pub fn len(&self) -> usize { self.len }
    pub fn is_empty(&self) -> bool { self.len == 0 }

    pub fn push(&mut self, record: Record) -> Result<(), Fault> {
        if N == 0 { return Err(Fault::InvalidConfig); }
        if self.len < N {
            let index = (self.oldest + self.len) % N;
            self.slots[index] = Some(record);
            self.len += 1;
        } else {
            self.slots[self.oldest] = Some(record);
            self.oldest = (self.oldest + 1) % N;
        }
        Ok(())
    }

    pub fn get(&self, index: usize) -> Option<Record> {
        if N == 0 || index >= self.len { return None; }
        self.slots[(self.oldest + index) % N]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Due {
    pub scheduled: u32,
    pub missed: u32,
    pub late_by: u32,
}

/// Periodic deadlines anchored to the original phase, not the polling time.
/// Caller must sample within the half-range of the wrapping u32 clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Periodic {
    last: u32,
    period: u32,
}

impl Periodic {
    pub fn new(start: u32, period: u32) -> Result<Self, Fault> {
        if period == 0 || period > MAX_INTERVAL { return Err(Fault::InvalidConfig); }
        Ok(Self { last: start, period })
    }

    pub fn poll(&mut self, now: u32) -> Result<Option<Due>, Fault> {
        let elapsed = now.wrapping_sub(self.last);
        if elapsed > MAX_INTERVAL { return Err(Fault::ClockAmbiguous); }
        let count = elapsed / self.period;
        if count == 0 { return Ok(None); }
        self.last = self.last.wrapping_add(count * self.period);
        Ok(Some(Due { scheduled: self.last, missed: count - 1,
                      late_by: now.wrapping_sub(self.last) }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hysteresis {
    low: u16,
    high: u16,
    enabled: bool,
}

impl Hysteresis {
    pub fn new(low: u16, high: u16) -> Result<Self, Fault> {
        if low >= high || high > RAW_MAX { return Err(Fault::InvalidConfig); }
        Ok(Self { low, high, enabled: false })
    }

    pub fn enabled(&self) -> bool { self.enabled }
    fn force_off(&mut self) { self.enabled = false; }

    pub fn update(&mut self, raw: u16) -> Result<bool, Fault> {
        if raw > RAW_MAX { return Err(Fault::Sensor); }
        if raw >= self.high { self.enabled = true; }
        else if raw <= self.low { self.enabled = false; }
        Ok(self.enabled)
    }
}

/// embedded-hal 1.0 defines digital pins; the numeric sensor interface is
/// application-specific because this lesson does not pretend to provide a universal ADC.
pub trait Sensor {
    type Error;
    fn read(&mut self) -> Result<u16, Self::Error>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Observation {
    pub raw: u16,
    pub enabled: bool,
}

pub struct Controller {
    band: Hysteresis,
    latched: bool,
}

impl Controller {
    pub fn new(low: u16, high: u16) -> Result<Self, Fault> {
        Ok(Self { band: Hysteresis::new(low, high)?, latched: false })
    }
    pub fn latched(&self) -> bool { self.latched }

    /// Explicit operator acknowledgement; the next step must recheck the interlock
    /// and sensor. This changes logical state only, not the physical output.
    pub fn acknowledge(&mut self) {
        self.latched = false;
        self.band.force_off();
    }

    // Fail closed logically; a failed output command is NEVER claimed to have
    // physically made an actuator safe. Report the output fault instead.
    fn fail_closed<O: OutputPin>(&mut self, output: &mut O, reason: Fault)
        -> Result<Observation, Fault>
    {
        self.latched = true;
        self.band.force_off();
        if output.set_low().is_err() { return Err(Fault::Output); }
        Err(reason)
    }

    pub fn step<S: Sensor, I: InputPin, O: OutputPin>(
        &mut self, sensor: &mut S, interlock: &mut I, output: &mut O
    ) -> Result<Observation, Fault> {
        if self.latched { return self.fail_closed(output, Fault::Latched); }
        if !matches!(interlock.is_high(), Ok(true)) {
            return self.fail_closed(output, Fault::Interlock);
        }
        let raw = match sensor.read() {
            Ok(raw) => raw,
            Err(_) => return self.fail_closed(output, Fault::Sensor),
        };
        let enabled = match self.band.update(raw) {
            Ok(value) => value,
            Err(reason) => return self.fail_closed(output, reason),
        };
        let command = if enabled { output.set_high() } else { output.set_low() };
        if command.is_err() { return self.fail_closed(output, Fault::Output); }
        Ok(Observation { raw, enabled })
    }
}

pub mod sim;
