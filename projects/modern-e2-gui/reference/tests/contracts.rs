use e2_signaldesk::*;
use e2_signaldesk::gui::controls;
use egui_kittest::{Harness,kittest::{Queryable,NodeT}};
use std::{cell::RefCell, rc::Rc, path::PathBuf, sync::atomic::{AtomicU64,Ordering}};

#[test]
fn step1_parse_strict_bounded_numeric_csv() {
    let rows=parse_samples(DEMO).unwrap();
    assert_eq!(rows.len(),8);
    assert_eq!(rows[3],Sample{time_s:3,value:33.0});
    assert_eq!(parse_samples("time_s,value\r\n2,-1.5\r\n8,1\r\n").unwrap().len(),2);
    for data in ["bad,value\n1,2","time_s,value\n","time_s,value\n1,NaN","time_s,value\n1,inf","time_s,value\n1,1001","time_s,value\n1,2,3","time_s,value\n1,2\n1,3","time_s,value\n2,2\n1,3","time_s,value\n-1,2"] {
        assert!(parse_samples(data).is_err(),"{data}");
    }
    assert_eq!(parse_samples(&"x".repeat(MAX_BYTES+1)),Err(Error::Limit));
    let long=format!("time_s,value\n{}",(0..=MAX_SAMPLES).map(|n|format!("{n},0\n")).collect::<String>());
    assert_eq!(parse_samples(&long),Err(Error::Limit));
}

#[test]
fn step2_statistics_are_calculated_from_real_values() {
    let m=Model::demo(Language::English);
    let s=summarize(&m.samples,30.0).unwrap();
    assert_eq!((s.count,s.min,s.max,s.alerts),(8,21.0,35.0,3));
    assert!((s.mean-28.375).abs()<1e-12);
    assert_eq!(summarize(&m.samples,33.0).unwrap().alerts,2);
    assert_eq!(summarize(&m.samples,35.0).unwrap().alerts,1);
    assert!(summarize(&[],30.0).is_none());
    assert!(summarize(&m.samples,f64::NAN).is_none());
}

#[test]
fn step3_invalid_actions_preserve_data_and_filter_is_reversible() {
    let mut m=Model::demo(Language::English);
    let original=m.clone();
    assert_eq!(m.apply(Action::Threshold(f64::NAN)),Err(Error::Range));
    assert_eq!(m,original);
    assert!(m.apply(Action::Replace(vec![])).is_err());
    assert_eq!(m,original);
    m.apply(Action::OnlyAlerts(true)).unwrap();
    assert_eq!(m.visible().iter().map(|s|s.time_s).collect::<Vec<_>>(),[3,4,6]);
    m.apply(Action::Threshold(34.0)).unwrap();
    assert_eq!(m.visible().len(),1);
    m.apply(Action::OnlyAlerts(false)).unwrap();
    assert_eq!(m.visible().len(),8);
    m.apply(Action::ToggleLanguage).unwrap();
    m.apply(Action::ResetDemo).unwrap();
    assert_eq!(m,Model::demo(Language::Chinese));
}

#[test]
fn step4_actual_egui_buttons_emit_actions() {
    let model=Model::demo(Language::English);
    let actions=Rc::new(RefCell::new(Vec::new()));
    let observed=Rc::clone(&actions);
    let mut draft=34.0;
    let mut harness=Harness::new_ui(move |ui| {
        observed.borrow_mut().extend(controls(ui,&model,&mut draft));
    });
    harness.get_by_label("Only alerts").click();
    harness.run();
    harness.get_by_label("Apply threshold").click();
    harness.run();
    harness.get_by_label("All samples").click();
    harness.run();
    assert_eq!(&*actions.borrow(),&[Action::OnlyAlerts(true),Action::Threshold(34.0),Action::OnlyAlerts(false)]);
}

static NEXT: AtomicU64=AtomicU64::new(0);
struct Temp(PathBuf);
impl Temp {
    fn new()->Self {
        let path=std::env::temp_dir().join(format!("signaldesk-{}-{}-{}",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos(),NEXT.fetch_add(1,Ordering::Relaxed)));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {fn drop(&mut self){let _=std::fs::remove_dir_all(&self.0);}}

#[test]
fn step5_real_file_import_export_preserves_original_and_filters() {
    let temp=Temp::new();
    let input=temp.0.join("示例 数据.csv");
    std::fs::write(&input,DEMO).unwrap();
    let mut model=Model::demo(Language::Chinese);
    model.apply(Action::Replace(read_csv(&input).unwrap())).unwrap();
    model.apply(Action::OnlyAlerts(true)).unwrap();
    let output=temp.0.join("visible.csv");
    export_csv(&output,&model).unwrap();
    assert_eq!(read_csv(&output).unwrap(),model.visible());
    let old=std::fs::read(&output).unwrap();
    assert_eq!(export_csv(&output,&model),Err(Error::File));
    assert_eq!(std::fs::read(&output).unwrap(),old);
    assert_eq!(std::fs::read_to_string(&input).unwrap(),DEMO);
    let before=model.clone();
    std::fs::write(&input,"time_s,value\n1,NaN").unwrap();
    let result=read_csv(&input).and_then(|rows|model.apply(Action::Replace(rows)));
    assert!(result.is_err());
    assert_eq!(model,before);
}
