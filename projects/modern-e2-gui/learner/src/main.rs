use e2_signaldesk::{Language, gui::SignalDesk};
use eframe::egui;

fn main() -> eframe::Result {
    let english=std::env::args().any(|v|v=="--english");
    let options=eframe::NativeOptions {
        viewport:egui::ViewportBuilder::default().with_inner_size([980.0,720.0]).with_min_inner_size([760.0,600.0]),
        renderer:eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native("SignalDesk",options,Box::new(move |cc| {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Ok(Box::new(SignalDesk::new(if english {Language::English} else {Language::Chinese})))
    }))
}
