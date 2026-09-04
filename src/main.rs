mod app;
mod config;

use app::WhichBowlApp;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Which Bowl is the Fish In?",
        options,
        Box::new(|cc| Ok(Box::new(WhichBowlApp::new(cc)))),
    )
}
