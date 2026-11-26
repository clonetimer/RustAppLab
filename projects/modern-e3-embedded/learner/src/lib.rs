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
        // TODO(1): bounded overwrite-oldest ring, handle N=0, retain logical order.
        let _ = (record, &self.slots, self.oldest);
        Err(Fault::Todo)
    }

    pub fn get(&self, index: usize) -> Option<Record> {
        // TODO(1b): map a logical index to the retained physical slot.
        let _ = index;
        None
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
        // TODO(2): wrapping time, phase preservation and count missed deadlines.
        let _ = (now, self.last, self.period);
        Err(Fault::Todo)
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
        // TODO(3): inclusive low/high thresholds; preserve state inside the band.
        let _ = (raw, self.low, self.high);
        Err(Fault::Todo)
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
        // TODO(4): use real embedded-hal traits; latch faults and command low;
        // distinguish failed output from confirmed successful command.
        let _ = (sensor, interlock, output);
        Err(Fault::Todo)
    }
}

pub mod sim;
