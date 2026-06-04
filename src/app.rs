use std::{fs, path::PathBuf};

use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

use crate::core::{
    compiler::{BuildResult, CompilerService},
    config::AppConfig,
    project::Project,
};
use crate::ui::{editor, file_tree, output_panel};
use crate::utils::paths;

pub struct LiteDevCppApp {
    project: Option<Project>,
    config: AppConfig,
    current_file: Option<PathBuf>,
    editor_text: String,
    dirty: bool,
    output: String,
    last_executable: Option<PathBuf>,
}

impl LiteDevCppApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_fonts(&cc.egui_ctx);
        let config = AppConfig::load().unwrap_or_default();
        Self {
            project: None,
            config,
            current_file: None,
            editor_text: String::new(),
            dirty: false,
            output: "Open a folder to begin.\n".to_owned(),
            last_executable: None,
        }
    }

    fn open_folder_dialog(&mut self) {
        if let Some(folder) = rfd::FileDialog::new().pick_folder() {
            self.open_folder(folder);
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
            self.push_output(
                "Current file has unsaved changes. Save it before opening another file.",
            );
            return;
        }

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

    fn save_current_file(&mut self) {
        let Some(path) = self.current_file.as_ref() else {
            self.push_output("No file is open.");
            return;
        };

        match std::fs::write(path, &self.editor_text) {
            Ok(()) => {
                self.dirty = false;
                self.push_output(format!("Saved file: {}", path.display()));
            }
            Err(err) => self.push_output(format!("Could not save file {}: {err}", path.display())),
        }
    }

    fn save_app_config(&mut self) {
        match self.config.save() {
            Ok(path) => self.push_output(format!("Saved config: {}", path.display())),
            Err(err) => self.push_output(format!("Could not save config: {err}")),
        }
    }

    fn build_current_file(&mut self) {
        let _ = self.build_current_file_inner();
    }

    fn build_and_run_current_file(&mut self) {
        if let Some(result) = self.build_current_file_inner() {
            if result.exit_code == 0 {
                self.run_last_executable();
            }
        }
    }

    fn build_current_file_inner(&mut self) -> Option<BuildResult> {
        if self.dirty {
            self.save_current_file();
        }

        let Some(file) = self.current_file.as_ref() else {
            self.push_output("Open a C/C++ file before building.");
            return None;
        };

        if !paths::is_supported_source(file) {
            self.push_output("The current file is not a supported C/C++ source file.");
            return None;
        }

        let service = CompilerService::new(self.config.compiler.clone());
        match service.build_current_file(file) {
            Ok(result) => {
                if result.exit_code == 0 {
                    self.last_executable = Some(result.executable.clone());
                } else {
                    self.last_executable = None;
                }
                self.append_build_result("Build", &result);
                Some(result)
            }
            Err(err) => {
                self.push_output(format!("Build failed: {err}"));
                None
            }
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

    fn push_output(&mut self, message: impl AsRef<str>) {
        self.output.push_str(message.as_ref());
        self.output.push('\n');
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
    }

    fn current_title(&self) -> String {
        match self.current_file.as_ref() {
            Some(path) if self.dirty => format!("{} *", path.display()),
            Some(path) => path.display().to_string(),
            None => "No file open".to_owned(),
        }
    }

    fn draw_toolbar(&mut self, ui: &mut egui::Ui) {
        if ui.button("Open Folder").clicked() {
            self.open_folder_dialog();
        }
        if ui.button("Save File").clicked() {
            self.save_current_file();
        }
        if ui.button("Build").clicked() {
            self.build_current_file();
        }
        if ui.button("Build & Run").clicked() {
            self.build_and_run_current_file();
        }
        if ui.button("Run").clicked() {
            self.run_last_executable();
        }
        if ui.button("Refresh").clicked() {
            self.refresh_project();
        }

        ui.separator();
        compiler_field(ui, "C", &mut self.config.compiler.c_compiler);
        compiler_field(ui, "C++", &mut self.config.compiler.cpp_compiler);
        if ui.button("Save Config").clicked() {
            self.save_app_config();
        }
    }
}

impl eframe::App for LiteDevCppApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| self.draw_toolbar(ui));
        });

        egui::SidePanel::left("file_tree")
            .resizable(true)
            .default_width(260.0)
            .width_range(180.0..=420.0)
            .show(ctx, |ui| {
                ui.heading("Project");
                ui.separator();
                let mut selected_file = None;
                file_tree::show(
                    ui,
                    self.project.as_ref(),
                    self.current_file.as_deref(),
                    &mut selected_file,
                );
                if let Some(path) = selected_file {
                    self.open_file(path);
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
