use std::path::{Path, PathBuf};

use eframe::egui;

#[derive(Debug, Clone)]
pub struct FileDialogState {
    pub kind: FileDialogKind,
    pub target: PathBuf,
    pub value: String,
}

#[derive(Debug, Clone, Copy)]
pub enum FileDialogKind {
    NewFile,
    NewFolder,
    Rename,
}

#[derive(Debug, Clone, Copy)]
pub enum DialogResponse {
    Apply,
    Cancel,
}

#[derive(Debug, Clone, Copy)]
pub enum UnsavedResponse {
    Save,
    Discard,
    Cancel,
}

pub fn show_file_dialog(
    ctx: &egui::Context,
    dialog: &mut FileDialogState,
) -> Option<DialogResponse> {
    let title = match dialog.kind {
        FileDialogKind::NewFile => "New File",
        FileDialogKind::NewFolder => "New Folder",
        FileDialogKind::Rename => "Rename",
    };
    let mut response = None;

    egui::Window::new(title)
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(dialog.target.display().to_string());
            ui.add(egui::TextEdit::singleline(&mut dialog.value).desired_width(280.0));
            ui.horizontal(|ui| {
                if ui.button("OK").clicked() {
                    response = Some(DialogResponse::Apply);
                }
                if ui.button("Cancel").clicked() {
                    response = Some(DialogResponse::Cancel);
                }
            });
        });

    response
}

pub fn show_delete_confirmation(ctx: &egui::Context, path: &Path) -> Option<DialogResponse> {
    let mut response = None;

    egui::Window::new("Delete")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Move this item to Trash?");
            ui.label(path.display().to_string());
            ui.horizontal(|ui| {
                if ui.button("Delete").clicked() {
                    response = Some(DialogResponse::Apply);
                }
                if ui.button("Cancel").clicked() {
                    response = Some(DialogResponse::Cancel);
                }
            });
        });

    response
}

pub fn show_unsaved_confirmation(ctx: &egui::Context) -> Option<UnsavedResponse> {
    let mut response = None;

    egui::Window::new("Unsaved Changes")
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Save the current file before continuing?");
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    response = Some(UnsavedResponse::Save);
                }
                if ui.button("Don't Save").clicked() {
                    response = Some(UnsavedResponse::Discard);
                }
                if ui.button("Cancel").clicked() {
                    response = Some(UnsavedResponse::Cancel);
                }
            });
        });

    response
}
