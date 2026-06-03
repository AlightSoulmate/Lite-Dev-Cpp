mod app;
mod core;
mod ui;
mod utils;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Lite Dev-C++")
            .with_inner_size([1180.0, 760.0])
            .with_min_inner_size([900.0, 580.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Lite Dev-C++",
        options,
        Box::new(|cc| Ok(Box::new(app::LiteDevCppApp::new(cc)))),
    )
}
