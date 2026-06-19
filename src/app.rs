use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::Duration,
};

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

use crate::core::{
    compiler::{BuildResult, CompilerService},
    config::AppConfig,
    file_ops,
    project::Project,
};
use crate::ui::{
    editor,
    file_dialog::{self, DialogResponse, FileDialogKind, FileDialogState, UnsavedResponse},
    file_tree::{self, FileTreeAction},
    output_panel,
};
use crate::utils::paths;

pub struct LiteDevCppApp {
    project: Option<Project>,
    config: AppConfig,
    current_file: Option<PathBuf>,
    editor_text: String,
    dirty: bool,
    output: String,
    last_executable: Option<PathBuf>,
    file_dialog: Option<FileDialogState>,
    delete_candidate: Option<PathBuf>,
    pending_action: Option<PendingAction>,
    allow_close: bool,
    build_receiver: Option<Receiver<io::Result<BuildResult>>>,
    run_after_build: bool,
}

enum PendingAction {
    OpenFolder(PathBuf),
    OpenFile(PathBuf),
    Close,
}

impl LiteDevCppApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_fonts(&cc.egui_ctx);
        let (config, output) = match AppConfig::load() {
            Ok(config) => (config, "Open a folder to begin.\n".to_owned()),
            Err(err) => (
                AppConfig::default(),
                format!("Could not load config; using defaults: {err}\n"),
            ),
        };
        Self {
            project: None,
            config,
            current_file: None,
            editor_text: String::new(),
            dirty: false,
            output,
            last_executable: None,
            file_dialog: None,
            delete_candidate: None,
            pending_action: None,
            allow_close: false,
            build_receiver: None,
            run_after_build: false,
        }
    }

    fn open_folder_dialog(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.request_action(PendingAction::OpenFolder(folder));
        }
    }

    fn open_folder(&mut self, folder: PathBuf) {
        match Project::open(folder.clone()) {
            Ok(project) => {
                self.project = Some(project);
                self.current_file = None;
                self.editor_text.clear();
                self.dirty = false;
                self.last_executable = None;
                self.push_output(format!("Opened folder: {}", folder.display()));
            }
            Err(err) => self.push_output(format!("Could not open folder: {err}")),
        }
    }

    fn open_file(&mut self, path: PathBuf) {
        if self.dirty {
            self.pending_action = Some(PendingAction::OpenFile(path));
            return;
        }

        self.open_file_now(path);
    }

    fn open_file_now(&mut self, path: PathBuf) {
        match std::fs::read_to_string(&path) {
            Ok(contents) => {
                self.current_file = Some(path.clone());
                self.editor_text = contents;
                self.dirty = false;
                self.last_executable = None;
                self.push_output(format!("Opened file: {}", path.display()));
            }
            Err(err) => self.push_output(format!("Could not open file {}: {err}", path.display())),
        }
    }

    fn save_current_file(&mut self) -> bool {
        let Some(path) = self.current_file.as_ref() else {
            self.push_output("No file is open.");
            return false;
        };

        match std::fs::write(path, &self.editor_text) {
            Ok(()) => {
                self.dirty = false;
                self.push_output(format!("Saved file: {}", path.display()));
                true
            }
            Err(err) => {
                self.push_output(format!("Could not save file {}: {err}", path.display()));
                false
            }
        }
    }

    fn save_app_config(&mut self) {
        match self.config.save() {
            Ok(path) => self.push_output(format!("Saved config: {}", path.display())),
            Err(err) => self.push_output(format!("Could not save config: {err}")),
        }
    }

    fn request_action(&mut self, action: PendingAction) {
        if self.dirty {
            self.pending_action = Some(action);
        } else {
            self.perform_action(action, None);
        }
    }

    fn perform_action(&mut self, action: PendingAction, ctx: Option<&egui::Context>) {
        match action {
            PendingAction::OpenFolder(folder) => self.open_folder(folder),
            PendingAction::OpenFile(path) => self.open_file_now(path),
            PendingAction::Close => {
                self.allow_close = true;
                if let Some(ctx) = ctx {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
    }

    fn build_current_file(&mut self) {
        self.start_build(false);
    }

    fn build_and_run_current_file(&mut self) {
        self.start_build(true);
    }

    fn start_build(&mut self, run_after_build: bool) {
        if self.build_receiver.is_some() {
            self.push_output("A build is already running.");
            return;
        }

        if self.dirty && !self.save_current_file() {
            self.push_output("Build cancelled because the file could not be saved.");
            return;
        }

        let Some(file) = self.current_file.clone() else {
            self.push_output("Open a C/C++ file before building.");
            return;
        };

        if !paths::is_supported_source(&file) {
            self.push_output("The current file is not a supported C/C++ source file.");
            return;
        }

        let service = CompilerService::new(self.config.compiler.clone());
        let (sender, receiver) = mpsc::channel();
        thread::spawn(move || {
            let _ = sender.send(service.build_current_file(&file));
        });

        self.last_executable = None;
        self.build_receiver = Some(receiver);
        self.run_after_build = run_after_build;
        self.push_output("Build started...");
    }

    fn poll_build(&mut self, ctx: &egui::Context) {
        let message = match self.build_receiver.as_ref() {
            Some(receiver) => receiver.try_recv(),
            None => return,
        };

        match message {
            Ok(result) => {
                self.build_receiver = None;
                self.finish_build(result);
            }
            Err(TryRecvError::Empty) => ctx.request_repaint_after(Duration::from_millis(50)),
            Err(TryRecvError::Disconnected) => {
                self.build_receiver = None;
                self.run_after_build = false;
                self.push_output("Build worker stopped unexpectedly.");
            }
        }
    }

    fn finish_build(&mut self, result: io::Result<BuildResult>) {
        let run_after_build = std::mem::take(&mut self.run_after_build);
        match result {
            Ok(result) => {
                if result.exit_code == 0 {
                    self.last_executable = Some(result.executable.clone());
                }
                self.append_build_result("Build", &result);
                if result.exit_code == 0 && run_after_build {
                    self.run_last_executable();
                }
            }
            Err(err) => self.push_output(format!("Build failed: {err}")),
        }
    }

    fn run_last_executable(&mut self) {
        let Some(executable) = self.last_executable.as_ref() else {
            self.push_output("Build the current file before running it.");
            return;
        };
        let Some(working_dir) = executable.parent() else {
            self.push_output("The executable path has no parent folder.");
            return;
        };

        let service = CompilerService::new(self.config.compiler.clone());
        match service.run_executable_in_terminal(executable, working_dir) {
            Ok(()) => self.push_output(format!("Launched in terminal: {}", executable.display())),
            Err(err) => self.push_output(format!("Run failed: {err}")),
        }
    }

    fn refresh_project(&mut self) {
        if let Some(project) = self.project.as_mut() {
            match project.refresh() {
                Ok(()) => self.push_output("File tree refreshed."),
                Err(err) => self.push_output(format!("Could not refresh file tree: {err}")),
            }
        }
    }

    fn handle_file_tree_action(&mut self, ctx: &egui::Context, action: FileTreeAction) {
        match action {
            FileTreeAction::Open(path) => self.open_file(path),
            FileTreeAction::Reveal(path) => match file_ops::reveal_in_finder(&path) {
                Ok(()) => self.push_output(format!("Revealed in Finder: {}", path.display())),
                Err(err) => self.push_output(format!("Could not reveal in Finder: {err}")),
            },
            FileTreeAction::CopyPath(path) => {
                ctx.copy_text(path.display().to_string());
                self.push_output(format!("Copied path: {}", path.display()));
            }
            FileTreeAction::NewFile(dir) => self.start_new_file(dir),
            FileTreeAction::NewFolder(dir) => self.start_new_folder(dir),
            FileTreeAction::Rename(path) => self.start_rename(path),
            FileTreeAction::Delete(path) => self.delete_candidate = Some(path),
            FileTreeAction::Refresh => self.refresh_project(),
        }
    }

    fn start_new_file(&mut self, dir: PathBuf) {
        self.file_dialog = Some(FileDialogState {
            kind: FileDialogKind::NewFile,
            value: file_ops::unique_child_name(&dir, "untitled.cpp"),
            target: dir,
        });
    }

    fn start_new_folder(&mut self, dir: PathBuf) {
        self.file_dialog = Some(FileDialogState {
            kind: FileDialogKind::NewFolder,
            value: file_ops::unique_child_name(&dir, "New Folder"),
            target: dir,
        });
    }

    fn start_rename(&mut self, path: PathBuf) {
        let value = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.file_dialog = Some(FileDialogState {
            kind: FileDialogKind::Rename,
            target: path,
            value,
        });
    }

    fn apply_file_dialog(&mut self) {
        let Some(dialog) = self.file_dialog.take() else {
            return;
        };
        let name = dialog.value.trim();
        if name.is_empty() {
            self.push_output("Name cannot be empty.");
            self.file_dialog = Some(dialog);
            return;
        }
        if name.contains('/') {
            self.push_output("Name cannot contain '/'.");
            self.file_dialog = Some(dialog);
            return;
        }

        let result = match dialog.kind {
            FileDialogKind::NewFile => self.create_file(&dialog.target, name),
            FileDialogKind::NewFolder => file_ops::create_folder(&dialog.target, name)
                .map(|path| format!("Created folder: {}", path.display())),
            FileDialogKind::Rename => self.rename_path(&dialog.target, name),
        };

        match result {
            Ok(message) => {
                self.push_output(message);
                self.refresh_project();
            }
            Err(err) => {
                self.push_output(err);
                self.file_dialog = Some(dialog);
            }
        }
    }

    fn create_file(&mut self, dir: &Path, name: &str) -> Result<String, String> {
        let path = file_ops::create_file(dir, name)?;
        self.open_file(path.clone());
        Ok(format!("Created file: {}", path.display()))
    }

    fn rename_path(&mut self, path: &Path, new_name: &str) -> Result<String, String> {
        let new_path = file_ops::rename_path(path, new_name)?;

        if let Some(current_file) = self.current_file.as_ref()
            && let Ok(relative) = current_file.strip_prefix(path)
        {
            self.current_file = Some(new_path.join(relative));
            self.last_executable = None;
        }

        Ok(format!(
            "Renamed {} to {}",
            path.display(),
            new_path.display()
        ))
    }

    fn confirm_delete_candidate(&mut self) {
        let Some(path) = self.delete_candidate.take() else {
            return;
        };

        match file_ops::move_to_trash(&path) {
            Ok(()) => {
                if self
                    .current_file
                    .as_deref()
                    .is_some_and(|file| file_ops::path_contains(&path, file))
                {
                    self.current_file = None;
                    self.editor_text.clear();
                    self.dirty = false;
                    self.last_executable = None;
                }
                self.push_output(format!("Moved to Trash: {}", path.display()));
                self.refresh_project();
            }
            Err(err) => self.push_output(format!("Could not move to Trash: {err}")),
        }
    }

    fn push_output(&mut self, message: impl AsRef<str>) {
        self.output.push_str(message.as_ref());
        self.output.push('\n');
        self.trim_output();
    }

    fn append_build_result(&mut self, label: &str, result: &BuildResult) {
        self.push_output(format!("== {label}: exit code {} ==", result.exit_code));
        if !result.stdout.trim().is_empty() {
            self.push_output("-- stdout --");
            self.output.push_str(&result.stdout);
            if !result.stdout.ends_with('\n') {
                self.output.push('\n');
            }
        }
        if !result.stderr.trim().is_empty() {
            self.push_output("-- stderr --");
            self.output.push_str(&result.stderr);
            if !result.stderr.ends_with('\n') {
                self.output.push('\n');
            }
        }
        if label == "Build" {
            self.push_output(format!("Executable: {}", result.executable.display()));
        }
        self.trim_output();
    }

    fn trim_output(&mut self) {
        const MAX_OUTPUT_BYTES: usize = 200_000;
        if self.output.len() <= MAX_OUTPUT_BYTES {
            return;
        }

        let mut start = self.output.len() - MAX_OUTPUT_BYTES;
        while !self.output.is_char_boundary(start) {
            start += 1;
        }
        self.output.drain(..start);
    }

    fn current_title(&self) -> String {
        match self.current_file.as_ref() {
            Some(path) if self.dirty => format!("{} *", path.display()),
            Some(path) => path.display().to_string(),
            None => "No file open".to_owned(),
        }
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui) {
        let building = self.build_receiver.is_some();
        let has_file = self.current_file.is_some();
        let can_build = self
            .current_file
            .as_deref()
            .is_some_and(paths::is_supported_source)
            && !building;

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            if ui.button("Open Folder…").clicked() {
                self.open_folder_dialog();
            }
            if ui
                .add_enabled(has_file, egui::Button::new("Save"))
                .clicked()
            {
                self.save_current_file();
            }
            ui.separator();
            if ui
                .add_enabled(can_build, egui::Button::new("Build"))
                .clicked()
            {
                self.build_current_file();
            }
            if ui
                .add_enabled(can_build, egui::Button::new("Build & Run"))
                .clicked()
            {
                self.build_and_run_current_file();
            }
            if ui
                .add_enabled(
                    self.last_executable.is_some() && !building,
                    egui::Button::new("Run"),
                )
                .clicked()
            {
                self.run_last_executable();
            }
            if ui
                .add_enabled(self.project.is_some(), egui::Button::new("Refresh"))
                .clicked()
            {
                self.refresh_project();
            }
            if building {
                ui.separator();
                ui.spinner();
                ui.label("Building…");
            }
        });

        ui.add_space(3.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 8.0;
            ui.strong("Compilers");
            compiler_field(ui, "C:", &mut self.config.compiler.c_compiler);
            compiler_field(ui, "C++:", &mut self.config.compiler.cpp_compiler);
            if ui.button("Save Config").clicked() {
                self.save_app_config();
            }
        });
    }

    fn draw_file_dialog(&mut self, ctx: &egui::Context) {
        let response = match self.file_dialog.as_mut() {
            Some(dialog) => file_dialog::show_file_dialog(ctx, dialog),
            None => None,
        };
        match response {
            Some(DialogResponse::Apply) => self.apply_file_dialog(),
            Some(DialogResponse::Cancel) => self.file_dialog = None,
            None => {}
        }
    }

    fn draw_delete_confirmation(&mut self, ctx: &egui::Context) {
        let response = self
            .delete_candidate
            .as_ref()
            .and_then(|path| file_dialog::show_delete_confirmation(ctx, path));
        match response {
            Some(DialogResponse::Apply) => self.confirm_delete_candidate(),
            Some(DialogResponse::Cancel) => self.delete_candidate = None,
            None => {}
        }
    }

    fn draw_unsaved_confirmation(&mut self, ctx: &egui::Context) {
        if self.pending_action.is_none() {
            return;
        }

        match file_dialog::show_unsaved_confirmation(ctx) {
            Some(UnsavedResponse::Save) => {
                if self.save_current_file()
                    && let Some(action) = self.pending_action.take()
                {
                    self.perform_action(action, Some(ctx));
                }
            }
            Some(UnsavedResponse::Discard) => {
                if let Some(action) = self.pending_action.take() {
                    self.perform_action(action, Some(ctx));
                }
            }
            Some(UnsavedResponse::Cancel) => self.pending_action = None,
            None => {}
        }
    }

    fn handle_close_request(&mut self, ctx: &egui::Context) {
        let close_requested = ctx.input(|input| input.viewport().close_requested());
        if close_requested && self.dirty && !self.allow_close {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.pending_action = Some(PendingAction::Close);
        }
    }
}

impl eframe::App for LiteDevCppApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_build(ctx);
        self.handle_close_request(ctx);

        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.add_space(4.0);
            self.draw_toolbar(ui);
            ui.add_space(4.0);
        });

        egui::SidePanel::left("file_tree")
            .resizable(true)
            .default_width(260.0)
            .width_range(180.0..=420.0)
            .show(ctx, |ui| {
                ui.heading("Project");
                ui.separator();
                let mut action = None;
                file_tree::show(
                    ui,
                    self.project.as_ref(),
                    self.current_file.as_deref(),
                    &mut action,
                );
                if let Some(action) = action {
                    self.handle_file_tree_action(ctx, action);
                }
            });

        egui::TopBottomPanel::bottom("output")
            .resizable(true)
            .default_height(180.0)
            .height_range(100.0..=360.0)
            .show(ctx, |ui| output_panel::show(ui, &mut self.output));

        egui::CentralPanel::default().show(ctx, |ui| {
            editor::show(
                ui,
                &self.current_title(),
                &mut self.editor_text,
                &mut self.dirty,
            );
        });

        self.draw_file_dialog(ctx);
        self.draw_delete_confirmation(ctx);
        self.draw_unsaved_confirmation(ctx);
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        // macOS handles Command-Q at the application level, before egui can
        // cancel the window close. Persist the active buffer so that shortcut
        // can never discard edits silently.
        if self.dirty {
            let _ = self.save_current_file();
        }
    }
}

fn compiler_field(ui: &mut egui::Ui, label: &str, value: &mut String) {
    ui.label(label);
    ui.add(egui::TextEdit::singleline(value).desired_width(90.0));
}

fn configure_fonts(ctx: &egui::Context) {
    let Some(font_bytes) = load_macos_system_cjk_font() else {
        return;
    };

    let mut fonts = FontDefinitions::default();
    fonts.font_data.insert(
        "macos_cjk".to_owned(),
        FontData::from_owned(font_bytes).into(),
    );

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .push("macos_cjk".to_owned());
    }

    ctx.set_fonts(fonts);
}

fn load_macos_system_cjk_font() -> Option<Vec<u8>> {
    let candidates = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/Supplemental/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Supplemental/Songti.ttc",
        "/System/Library/Fonts/CJKSymbolsFallback.ttc",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
        "/System/Library/Fonts/Supplemental/NISC18030.ttf",
    ];

    candidates.iter().find_map(|path| fs::read(path).ok())
}
