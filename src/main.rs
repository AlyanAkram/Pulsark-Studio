mod app;
mod file_tree;
mod explorer;
mod editor;
mod terminal;
mod ai_panel;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Pulsark Studio",
        options,
        Box::new(|_cc| Box::new(app::MyApp::default())),
    )
}