use e3_embedded_lab::{Controller, Due, Fault, History, Hysteresis, Observation, Periodic, Record, sim::*};

fn record(tick: u32) -> Record {
    Record { scheduled: tick, raw: tick as u16, enabled: false, missed: 0 }
}

#[test]
fn step1_fixed_capacity_history_preserves_order_and_zero_capacity() {
    let mut history = History::<3>::new();
    assert!(history.is_empty());
    assert_eq!(history.get(0), None);
    for tick in 1..=5 { history.push(record(tick)).unwrap(); }
    assert_eq!(history.len(), 3);
    assert_eq!((history.get(0), history.get(1), history.get(2)),
               (Some(record(3)), Some(record(4)), Some(record(5))));
    assert_eq!(history.get(3), None);
    let mut zero = History::<0>::new();
    assert_eq!(zero.push(record(1)), Err(Fault::InvalidConfig));
    assert_eq!(zero.get(0), None);
    let mut one = History::<1>::new();
    one.push(record(1)).unwrap();
    one.push(record(2)).unwrap();
    assert_eq!(one.get(0), Some(record(2)));
}

#[test]
fn step2_wrapping_schedule_preserves_phase_and_counts_misses() {
    assert!(Periodic::new(0, 0).is_err());
    assert!(Periodic::new(0, 1 << 31).is_err());
    let mut schedule = Periodic::new(100, 10).unwrap();
    assert_eq!(schedule.poll(109), Ok(None));
    assert_eq!(schedule.poll(110), Ok(Some(Due {scheduled:110,missed:0,late_by:0})));
    assert_eq!(schedule.poll(145), Ok(Some(Due {scheduled:140,missed:2,late_by:5})));
    assert_eq!(schedule.poll(149), Ok(None));
    let unchanged = schedule;
    assert_eq!(schedule.poll(130), Err(Fault::ClockAmbiguous));
    assert_eq!(schedule, unchanged);
    assert_eq!(schedule.poll(150).unwrap().unwrap().scheduled, 150);
    let mut wrap = Periodic::new(u32::MAX - 2, 4).unwrap();
    assert_eq!(wrap.poll(0), Ok(None));
    assert_eq!(wrap.poll(1), Ok(Some(Due {scheduled:1,missed:0,late_by:0})));
    assert_eq!(wrap.poll(10), Ok(Some(Due {scheduled:9,missed:1,late_by:1})));
}

#[test]
fn step3_hysteresis_has_inclusive_edges_and_retains_state_inside_band() {
    assert!(Hysteresis::new(2000,2000).is_err());
    assert!(Hysteresis::new(1600,4096).is_err());
    let mut band = Hysteresis::new(1600,2000).unwrap();
    for (raw, enabled) in [(1700,false),(2000,true),(1800,true),(1600,false),(1900,false),(4095,true)] {
        assert_eq!(band.update(raw), Ok(enabled));
    }
    let unchanged = band;
    assert_eq!(band.update(4096), Err(Fault::Sensor));
    assert_eq!(band, unchanged);
}

#[test]
fn step4_real_hal_traits_fault_latch_and_failed_safe_command() {
    let mut controller = Controller::new(1600,2000).unwrap();
    let mut sensor = SimSensor {raw:2500,fail:false};
    let mut interlock = SimInput {high:true,fail:false};
    let mut pin = SimOutput::new();
    assert_eq!(controller.step(&mut sensor,&mut interlock,&mut pin),Ok(Observation{raw:2500,enabled:true}));
    assert!(pin.high);
    interlock.high=false;
    assert_eq!(controller.step(&mut sensor,&mut interlock,&mut pin),Err(Fault::Interlock));
    assert!(!pin.high && controller.latched());
    interlock.high=true;
    assert_eq!(controller.step(&mut sensor,&mut interlock,&mut pin),Err(Fault::Latched));
    assert!(!pin.high);
    controller.acknowledge();
    sensor.fail=true;
    assert_eq!(controller.step(&mut sensor,&mut interlock,&mut pin),Err(Fault::Sensor));
    controller.acknowledge();
    sensor.fail=false;
    sensor.raw=4096;
    assert_eq!(controller.step(&mut sensor,&mut interlock,&mut pin),Err(Fault::Sensor));
    controller.acknowledge();
    sensor.raw=2500;
    interlock.fail=true;
    assert_eq!(controller.step(&mut sensor,&mut interlock,&mut pin),Err(Fault::Interlock));
    controller.acknowledge();
    interlock.fail=false;
    pin.fail_high=true;
    assert_eq!(controller.step(&mut sensor,&mut interlock,&mut pin),Err(Fault::Output));
    assert!(controller.latched() && !pin.high);
    controller.acknowledge();
    pin.fail_high=false;
    assert!(controller.step(&mut sensor,&mut interlock,&mut pin).unwrap().enabled);
    pin.fail_low=true;
    interlock.high=false;
    assert_eq!(controller.step(&mut sensor,&mut interlock,&mut pin),Err(Fault::Output));
    assert!(controller.latched());
    assert!(pin.high, "failed low command must NOT be claimed safe");
}

#[test]
fn step5_same_core_drives_a_host_scheduled_trace() {
    let mut schedule=Periodic::new(0,2).unwrap();
    let mut history=History::<4>::new();
    let mut controller=Controller::new(1600,2000).unwrap();
    let mut sensor=SimSensor{raw:0,fail:false};
    let mut interlock=SimInput{high:true,fail:false};
    let mut output=SimOutput::new();
    for (tick,raw) in [(2,1500),(4,2100),(9,1800),(10,1400)] {
        let due=schedule.poll(tick).unwrap().unwrap();
        sensor.raw=raw;
        let result=controller.step(&mut sensor,&mut interlock,&mut output).unwrap();
        history.push(Record{scheduled:due.scheduled,raw:result.raw,enabled:result.enabled,missed:due.missed}).unwrap();
    }
    assert_eq!((history.get(0).unwrap().enabled,history.get(1).unwrap().enabled,
                history.get(2).unwrap().enabled,history.get(3).unwrap().enabled),
               (false,true,true,false));
    assert_eq!(history.get(2).unwrap(),Record{scheduled:8,raw:1800,enabled:true,missed:1});
    assert!(!output.high);
}
