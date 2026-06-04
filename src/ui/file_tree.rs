use std::path::{Path, PathBuf};

use eframe::egui;

use crate::core::project::{FileNode, Project};

#[derive(Debug, Clone)]
pub enum FileTreeAction {
    Open(PathBuf),
    Reveal(PathBuf),
    CopyPath(PathBuf),
    NewFile(PathBuf),
    NewFolder(PathBuf),
    Rename(PathBuf),
    Delete(PathBuf),
    Refresh,
}

pub fn show(
    ui: &mut egui::Ui,
    project: Option<&Project>,
    current_file: Option<&Path>,
    action: &mut Option<FileTreeAction>,
) {
    let Some(project) = project else {
        ui.label("No folder open.");
        return;
    };

    let response = ui.label(project.root().display().to_string());
    response.context_menu(|ui| {
        folder_menu(ui, project.root(), true, action);
    });
    ui.separator();

    for node in project.nodes() {
        show_node(ui, node, current_file, action);
    }
}

fn show_node(
    ui: &mut egui::Ui,
    node: &FileNode,
    current_file: Option<&Path>,
    action: &mut Option<FileTreeAction>,
) {
    if node.is_dir {
        let response = egui::CollapsingHeader::new(&node.name)
            .default_open(false)
            .show(ui, |ui| {
                for child in &node.children {
                    show_node(ui, child, current_file, action);
                }
            });
        response.header_response.context_menu(|ui| {
            folder_menu(ui, &node.path, false, action);
        });
        return;
    }

    let selected = current_file.is_some_and(|path| path == node.path.as_path());
    let response = ui.selectable_label(selected, &node.name);
    if response.clicked() {
        *action = Some(FileTreeAction::Open(node.path.clone()));
    }
    response.context_menu(|ui| file_menu(ui, &node.path, action));
}

fn file_menu(ui: &mut egui::Ui, path: &Path, action: &mut Option<FileTreeAction>) {
    menu_item(ui, "Open", FileTreeAction::Open(path.to_path_buf()), action);
    ui.separator();
    menu_item(
        ui,
        "Reveal in Finder",
        FileTreeAction::Reveal(path.to_path_buf()),
        action,
    );
    menu_item(
        ui,
        "Copy Path",
        FileTreeAction::CopyPath(path.to_path_buf()),
        action,
    );
    ui.separator();
    menu_item(
        ui,
        "Rename",
        FileTreeAction::Rename(path.to_path_buf()),
        action,
    );
    menu_item(
        ui,
        "Delete",
        FileTreeAction::Delete(path.to_path_buf()),
        action,
    );
}

fn folder_menu(
    ui: &mut egui::Ui,
    path: &Path,
    is_project_root: bool,
    action: &mut Option<FileTreeAction>,
) {
    menu_item(
        ui,
        "Reveal in Finder",
        FileTreeAction::Reveal(path.to_path_buf()),
        action,
    );
    menu_item(
        ui,
        "Copy Path",
        FileTreeAction::CopyPath(path.to_path_buf()),
        action,
    );
    ui.separator();
    menu_item(
        ui,
        "New File",
        FileTreeAction::NewFile(path.to_path_buf()),
        action,
    );
    menu_item(
        ui,
        "New Folder",
        FileTreeAction::NewFolder(path.to_path_buf()),
        action,
    );
    ui.separator();
    menu_item(ui, "Refresh", FileTreeAction::Refresh, action);

    if !is_project_root {
        ui.separator();
        menu_item(
            ui,
            "Rename",
            FileTreeAction::Rename(path.to_path_buf()),
            action,
        );
        menu_item(
            ui,
            "Delete",
            FileTreeAction::Delete(path.to_path_buf()),
            action,
        );
    }
}

fn menu_item(
    ui: &mut egui::Ui,
    label: &str,
    next: FileTreeAction,
    action: &mut Option<FileTreeAction>,
) {
    if ui.button(label).clicked() {
        *action = Some(next);
        ui.close_menu();
    }
}
