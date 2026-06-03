use eframe::egui;

pub fn show(ui: &mut egui::Ui, output: &mut String) {
    ui.horizontal(|ui| {
        ui.heading("Output");
        if ui.button("Clear").clicked() {
            output.clear();
        }
    });
    ui.separator();
    egui::ScrollArea::vertical()
        .stick_to_bottom(true)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(output)
                    .font(egui::TextStyle::Monospace)
                    .desired_width(f32::INFINITY)
                    .desired_rows(8)
                    .interactive(false),
            );
        });
}
