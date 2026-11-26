use e3_embedded_lab::{Controller, History, Periodic, Record, sim::*};
fn main() {
    let mut schedule=Periodic::new(0,2).unwrap();
    let mut controller=Controller::new(1600,2000).unwrap();
    let mut history=History::<4>::new();
    let mut sensor=SimSensor{raw:0,fail:false};
    let mut interlock=SimInput{high:true,fail:false};
    let mut output=SimOutput::new();
    for (tick,raw) in [(2,1500),(4,2100),(9,1800),(10,1400)] {
        let due=schedule.poll(tick).unwrap().unwrap();
        sensor.raw=raw;
        let result=controller.step(&mut sensor,&mut interlock,&mut output).unwrap();
        history.push(Record{scheduled:due.scheduled,raw:result.raw,enabled:result.enabled,missed:due.missed}).unwrap();
    }
    for index in 0..history.len() {
        let row=history.get(index).unwrap();
        println!("scheduled={} raw={} enabled={} missed={}",row.scheduled,row.raw,row.enabled as u8,row.missed);
    }
    println!("host fixture only; not a physical timing test");
}
