#![no_std]
#![no_main]
//! Actual Cortex-M3 image, with a real SysTick interrupt under QEMU.
//! Sensor/interlock/actuator fixtures remain explicitly simulated HAL devices.
use core::sync::atomic::{AtomicU32, Ordering};
use cortex_m::{asm, peripheral::syst::SystClkSource, Peripherals};
use cortex_m_rt::{entry, exception};
use cortex_m_semihosting::{debug, hprintln};
use e3_embedded_lab::{Controller, Fault, History, Periodic, Record, sim::*};

static TICKS: AtomicU32 = AtomicU32::new(0);

#[exception]
fn SysTick() {
    TICKS.fetch_add(1, Ordering::Relaxed);
}

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().expect("core peripherals");
    let mut schedule = Periodic::new(0, 2).unwrap();
    let mut controller = Controller::new(1600, 2000).unwrap();
    let mut sensor = SimSensor { raw: 0, fail: false };
    let mut interlock = SimInput { high: true, fail: false };
    let mut actuator = SimOutput::new();
    let mut history = History::<4>::new();
    let script = [1500, 2100, 1800, 1400];

    peripherals.SYST.set_clock_source(SystClkSource::Core);
    // Count actual interrupts, not assumed milliseconds. QEMU virtual timing
    // and the board clock are not a physical deadline measurement.
    peripherals.SYST.set_reload(50_000);
    peripherals.SYST.clear_current();
    peripherals.SYST.enable_interrupt();
    peripherals.SYST.enable_counter();

    while history.len() < script.len() {
        let now = TICKS.load(Ordering::Relaxed);
        if let Some(due) = schedule.poll(now).unwrap() {
            sensor.raw = script[history.len()];
            let observed = controller.step(&mut sensor, &mut interlock, &mut actuator).unwrap();
            history.push(Record { scheduled: due.scheduled, raw: observed.raw,
                                 enabled: observed.enabled, missed: due.missed }).unwrap();
        }
        if history.len() < script.len() { asm::wfi(); }
    }
    peripherals.SYST.disable_interrupt();
    peripherals.SYST.disable_counter();

    let interrupts = TICKS.load(Ordering::Relaxed);
    assert!(interrupts >= 8);
    let _ = hprintln!("timer_irq_count={}", interrupts);
    for index in 0..history.len() {
        let record = history.get(index).unwrap();
        let _ = hprintln!("sample={} scheduled={} raw={} enabled={} missed={}",
            index, record.scheduled, record.raw, record.enabled as u8, record.missed);
    }

    // Demonstrate a safe command after a simulated interlock opens.
    sensor.raw = 3000;
    assert!(controller.step(&mut sensor, &mut interlock, &mut actuator).unwrap().enabled);
    assert!(actuator.high);
    interlock.high = false;
    assert_eq!(controller.step(&mut sensor, &mut interlock, &mut actuator), Err(Fault::Interlock));
    assert!(controller.latched());
    assert!(!actuator.high);
    let _ = hprintln!("simulated_interlock=latched output_command=low");
    let _ = hprintln!("E3_QEMU_PASS");
    debug::exit(debug::EXIT_SUCCESS);
    loop { asm::wfi(); }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    let _ = hprintln!("E3_PANIC");
    debug::exit(debug::EXIT_FAILURE);
    loop { asm::wfi(); }
}
