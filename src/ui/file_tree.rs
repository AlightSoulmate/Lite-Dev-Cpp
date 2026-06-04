use std::path::{Path, PathBuf};

use eframe::egui;

use crate::core::project::{FileNode, Project};

pub fn show(
    ui: &mut egui::Ui,
    project: Option<&Project>,
    current_file: Option<&Path>,
    selected_file: &mut Option<PathBuf>,
) {
    let Some(project) = project else {
        ui.label("No folder open.");
        return;
    };

    ui.label(project.root().display().to_string());
    ui.separator();

    for node in project.nodes() {
        show_node(ui, node, current_file, selected_file);
    }
}

fn show_node(
    ui: &mut egui::Ui,
    node: &FileNode,
    current_file: Option<&Path>,
    selected_file: &mut Option<PathBuf>,
) {
    if node.is_dir {
        egui::CollapsingHeader::new(&node.name)
            .default_open(false)
            .show(ui, |ui| {
                for child in &node.children {
                    show_node(ui, child, current_file, selected_file);
                }
            });
        return;
    }

    let selected = current_file.is_some_and(|path| path == node.path.as_path());
    let response = ui.selectable_label(selected, &node.name);
    if response.clicked() {
        *selected_file = Some(node.path.clone());
    }
}
