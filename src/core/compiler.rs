use std::{
    collections::hash_map::DefaultHasher,
    env, fs,
    hash::{Hash, Hasher},
    io,
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

    pub fn build_current_file(&self, source: &Path) -> io::Result<BuildResult> {
        let compiler = self.compiler_for_source(source)?;
        let source_dir = source_parent(source)?;
        let executable = build_output_path(source)?;

        let output = Command::new(compiler)
            .arg(source)
            .arg("-o")
            .arg(&executable)
            .current_dir(source_dir)
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
        Ok(())
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

fn build_output_path(source: &Path) -> io::Result<PathBuf> {
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    let build_dir = env::temp_dir()
        .join("lite-dev-cpp")
        .join(format!("{:016x}", hasher.finish()));
    fs::create_dir_all(&build_dir)?;

    let executable_name = source
        .file_stem()
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| std::ffi::OsStr::new("program"));
    Ok(build_dir.join(executable_name))
}

fn shell_quote(path: &Path) -> String {
    let value = path.to_string_lossy();
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn apple_script_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn source_parent(source: &Path) -> io::Result<&Path> {
    source
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "source file has no parent"))
}

#[cfg(test)]
mod tests {
    use super::{apple_script_string, shell_quote};
    use std::path::Path;

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(
            shell_quote(Path::new("/tmp/it's here")),
            "'/tmp/it'\\''s here'"
        );
    }

    #[test]
    fn apple_script_string_escapes_special_characters() {
        assert_eq!(apple_script_string("a\\b\"c"), "\"a\\\\b\\\"c\"");
    }
}
