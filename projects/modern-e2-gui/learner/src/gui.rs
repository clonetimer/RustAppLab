//! Supplied rendering infrastructure; learners implement real controls (TODO 4).
use eframe::egui::{self, Color32, Stroke, RichText};
use crate::{Action, Error, Language, Model, read_csv, export_csv, summarize};
use std::path::Path;

pub struct SignalDesk {
    pub model: Model,
    draft: f64,
    import_path: String,
    export_path: String,
    status: Result<&'static str, Error>,
    title: String,
}

impl SignalDesk {
    pub fn new(language: Language) -> Self {
        Self { model:Model::demo(language), draft:30.0, import_path:String::new(),
            export_path:String::new(), status:Ok("ready"), title:String::new() }
    }

    fn dispatch(&mut self, action: Action) {
        self.status = self.model.apply(action).map(|_| "ready");
        self.draft = self.model.threshold;
    }
}

fn tr(lang: Language, en: &'static str, zh: &'static str) -> &'static str { lang.text(en, zh) }

pub fn controls(ui: &mut egui::Ui, model: &Model, draft: &mut f64) -> Vec<Action> {
    // TODO(4): real egui slider and buttons. Return model Actions, without directly editing data.
    let _ = (model, draft);
    ui.label("Implement the control panel / 实现控件");
    Vec::new()
}

fn chart(ui: &mut egui::Ui, model: &Model) {
    let width = ui.available_width().max(100.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, 210.0), egui::Sense::hover());
    let plot = rect.shrink2(egui::vec2(24.0,16.0));
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 6.0, Color32::from_rgb(20,29,43));
    if model.samples.is_empty() { return; }
    let lo = model.samples.iter().fold(model.threshold, |v,s| v.min(s.value));
    let hi = model.samples.iter().fold(model.threshold, |v,s| v.max(s.value));
    let span = (hi-lo).max(1.0);
    let t0 = model.samples[0].time_s;
    let duration = (model.samples.last().unwrap().time_s-t0).max(1);
    let position = |time:u32,value:f64| egui::pos2(
        plot.left()+plot.width()*((time-t0) as f32/duration as f32),
        plot.bottom()-plot.height()*((value-lo)/span) as f32);
    for k in 0..=4 {
        let y = plot.top()+plot.height()*k as f32/4.0;
        painter.line_segment([egui::pos2(plot.left(),y),egui::pos2(plot.right(),y)],
            Stroke::new(1.0,Color32::from_rgb(48,59,76)));
    }
    let threshold = position(t0,model.threshold).y;
    painter.line_segment([egui::pos2(plot.left(),threshold),egui::pos2(plot.right(),threshold)],
        Stroke::new(2.0,Color32::from_rgb(242,175,76)));
    for pair in model.samples.windows(2) {
        painter.line_segment([position(pair[0].time_s,pair[0].value),position(pair[1].time_s,pair[1].value)],
            Stroke::new(2.0,Color32::from_rgb(81,191,210)));
    }
    for row in &model.samples {
        let alert = row.value >= model.threshold;
        painter.circle_filled(position(row.time_s,row.value),if alert {4.5} else {3.0},
            if alert {Color32::from_rgb(242,175,76)} else {Color32::from_rgb(81,191,210)});
    }
}

impl eframe::App for SignalDesk {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        // Genuine native key events and the clickable controls share the same state reducer.
        let pressed = |key| ctx.input(|i| ctrl_pressed(&i.events, key));
        if pressed(egui::Key::F) { self.dispatch(Action::OnlyAlerts(!self.model.only_alerts)); }
        if pressed(egui::Key::R) { self.dispatch(Action::ResetDemo); }
        if pressed(egui::Key::L) { self.dispatch(Action::ToggleLanguage); }
        if pressed(egui::Key::Q) { ctx.send_viewport_cmd(egui::ViewportCommand::Close); }
        let language = self.model.language;
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.heading(RichText::new(tr(language,"SignalDesk | Local data explorer","SignalDesk | 本地数据分析台")).size(23.0));
                if ui.button(tr(language,"中文","English")).clicked() {
                    self.dispatch(Action::ToggleLanguage);
                }
            });
            ui.label(tr(language,"Numeric CSV • time_s,value • local files only","数值 CSV · time_s,value · 仅本地文件"));
            ui.separator();
            for action in controls(ui,&self.model,&mut self.draft) { self.dispatch(action); }
            if let Some(s) = summarize(&self.model.samples,self.model.threshold) {
                ui.horizontal_wrapped(|ui| {
                    ui.label(format!("{}: {}",tr(language,"Samples","样本"),s.count));
                    ui.label(format!("{}: {:.2}",tr(language,"Mean","均值"),s.mean));
                    ui.label(format!("{}: {:.2} / {:.2}",tr(language,"Min / Max","最小 / 最大"),s.min,s.max));
                    ui.label(RichText::new(format!("{}: {}",tr(language,"Alerts","告警"),s.alerts)).color(Color32::from_rgb(242,175,76)));
                });
            }
            chart(ui,&self.model);
            ui.small(tr(language,"Cyan: signal  •  Amber: threshold / alert   |   x: time (s), y: value","青色：数据曲线  ·  橙色：阈值与告警   |   横轴：时间(s)，纵轴：数值"));
            ui.separator();
            ui.horizontal(|ui| {
                let label=ui.label(tr(language,"Import path","导入路径"));
                ui.text_edit_singleline(&mut self.import_path).labelled_by(label.id);
                if ui.button(tr(language,"Load CSV","加载 CSV")).clicked() {
                    self.status=read_csv(Path::new(&self.import_path)).and_then(|rows|self.model.apply(Action::Replace(rows))).map(|_|"loaded");
                }
            });
            ui.horizontal(|ui| {
                let label=ui.label(tr(language,"Export path","导出路径"));
                ui.text_edit_singleline(&mut self.export_path).labelled_by(label.id);
                if ui.button(tr(language,"Export visible CSV","导出可见数据")).clicked() {
                    self.status=export_csv(Path::new(&self.export_path),&self.model).map(|_|"exported");
                }
            });
            let visible = self.model.visible();
            ui.label(RichText::new(format!("{} {}/{}",tr(language,"Visible","可见"),visible.len(),self.model.samples.len())).strong());
            egui::ScrollArea::vertical().max_height(150.0).show(ui, |ui| {
                egui::Grid::new("samples").striped(true).num_columns(3).show(ui,|ui| {
                    ui.strong("time_s"); ui.strong("value"); ui.strong(tr(language,"State","状态")); ui.end_row();
                    for row in &visible {
                        ui.label(row.time_s.to_string());
                        ui.label(format!("{:.2}",row.value));
                        ui.label(if row.value >= self.model.threshold { tr(language,"Alert","告警") } else { tr(language,"Normal","正常") });
                        ui.end_row();
                    }
                });
            });
            match self.status {
                Ok(message) => { ui.small(format!("{}: {message}",tr(language,"Status","状态"))); }
                Err(error) => { ui.colored_label(Color32::LIGHT_RED,format!("{}: {error:?}",tr(language,"Error (data preserved on failed import)","错误（导入失败保留原数据）"))); }
            }
            ui.small("Ctrl+F: filter | Ctrl+R: reset | Ctrl+L: language | Ctrl+Q: quit");
        });
        let count = self.model.samples.len();
        let visible = self.model.visible().len();
        let alerts = self.model.samples.iter().filter(|s|s.value>=self.model.threshold).count();
        let mode = if self.model.only_alerts {"alerts"} else {"all"};
        let lang = if self.model.language==Language::Chinese {"zh"} else {"en"};
        let title=format!("SignalDesk | rows={count} alerts={alerts} visible={visible} mode={mode} lang={lang}");
        if title!=self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title=title;
        }
    }
}

// Match the modifier snapshot on the key-down event, not the frame's final
// modifier state: Ctrl may already have been released before egui paints.
fn ctrl_pressed(events: &[egui::Event], key: egui::Key) -> bool {
    events.iter().any(|event| {
        matches!(
            event,
            egui::Event::Key {
                key: event_key,
                pressed: true,
                repeat: false,
                modifiers,
                ..
            } if *event_key == key && modifiers.ctrl
        )
    })
}

#[cfg(test)]
mod shortcut_tests {
    use super::*;

    fn key_event(key: egui::Key, pressed: bool, repeat: bool, modifiers: egui::Modifiers) -> egui::Event {
        egui::Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat,
            modifiers,
        }
    }

    #[test]
    fn ctrl_chord_uses_event_modifiers_even_if_released_in_same_frame() {
        for key in [egui::Key::F, egui::Key::R, egui::Key::L, egui::Key::Q] {
            let events = [
                key_event(key, true, false, egui::Modifiers::CTRL),
                key_event(key, false, false, egui::Modifiers::NONE),
            ];
            assert!(ctrl_pressed(&events, key));
            assert!(!ctrl_pressed(&events[1..], key));
            assert!(!ctrl_pressed(&[key_event(key, true, false, egui::Modifiers::NONE)], key));
            assert!(!ctrl_pressed(&[key_event(key, true, true, egui::Modifiers::CTRL)], key));
        }
        assert!(!ctrl_pressed(&[key_event(egui::Key::F, true, false, egui::Modifiers::CTRL)], egui::Key::L));
    }
}
