use std::{
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use crate::{core::config::CompilerConfig, utils::paths};

#[derive(Debug, Clone)]
pub struct CompilerService {
    config: CompilerConfig,
}

#[derive(Debug, Clone)]
pub struct BuildResult {
    pub executable: PathBuf,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl CompilerService {
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    pub fn build_current_file(
        &self,
        project_root: &Path,
        source: &Path,
    ) -> io::Result<BuildResult> {
        let compiler = self.compiler_for_source(source)?;
        let build_dir = project_root.join("build");
        fs::create_dir_all(&build_dir)?;
        let executable = output_executable_path(&build_dir, source);

        let output = Command::new(compiler)
            .arg(source)
            .arg("-o")
            .arg(&executable)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        Ok(BuildResult {
            executable,
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            exit_code: output.status.code().unwrap_or(-1),
        })
    }

    pub fn run_executable_in_terminal(
        &self,
        executable: &Path,
        working_dir: &Path,
    ) -> io::Result<()> {
        #[cfg(target_os = "macos")]
        {
            let command = format!(
                "cd {} && {} ; echo ; echo '[Lite Dev-C++] Process finished. Press Enter to close.' ; read",
                shell_quote(working_dir),
                shell_quote(executable),
            );
            let script = format!(
                "tell application \"Terminal\"\n  activate\n  do script {}\nend tell",
                apple_script_string(&command)
            );
            Command::new("osascript")
                .arg("-e")
                .arg(script)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            return Ok(());
        }

        #[cfg(target_os = "windows")]
        {
            let command = format!(
                "cd /d {} && {}",
                windows_cmd_quote(working_dir),
                windows_cmd_quote(executable)
            );
            Command::new("cmd")
                .arg("/C")
                .arg("start")
                .arg("Lite Dev-C++")
                .arg("cmd")
                .arg("/K")
                .arg(command)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            return Ok(());
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Command::new("x-terminal-emulator")
                .arg("-e")
                .arg(executable)
                .current_dir(working_dir)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()?;
            Ok(())
        }
    }

    fn compiler_for_source(&self, source: &Path) -> io::Result<&str> {
        if paths::is_c_source(source) {
            return Ok(&self.config.c_compiler);
        }
        if paths::is_cpp_source(source) {
            return Ok(&self.config.cpp_compiler);
        }

        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "unsupported source extension",
        ))
    }
}

#[cfg(target_os = "macos")]
fn shell_quote(path: &Path) -> String {
    let value = path.to_string_lossy();
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
fn apple_script_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[cfg(target_os = "windows")]
fn windows_cmd_quote(path: &Path) -> String {
    format!("\"{}\"", path.display())
}

fn output_executable_path(build_dir: &Path, source: &Path) -> PathBuf {
    let stem = source.file_stem().unwrap_or_default();
    let mut file_name = OsString::from(stem);
    if cfg!(windows) {
        file_name.push(".exe");
    }
    build_dir.join(file_name)
}
