use std::{
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub compiler: CompilerConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    #[serde(default = "default_c_compiler")]
    pub c_compiler: String,
    #[serde(default = "default_cpp_compiler")]
    pub cpp_compiler: String,
}

impl AppConfig {
    pub const FILE_NAME: &'static str = "lite-dev-cpp.toml";

    pub fn load_from_project(project_root: &Path) -> io::Result<Self> {
        let path = project_root.join(Self::FILE_NAME);
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid project config: {err}"),
            )
        })
    }

    pub fn save_to_project(&self, project_root: &Path) -> io::Result<PathBuf> {
        let path = project_root.join(Self::FILE_NAME);
        let contents = toml::to_string_pretty(self).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("could not serialize config: {err}"),
            )
        })?;
        fs::write(&path, contents)?;
        Ok(path)
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            compiler: CompilerConfig::default(),
        }
    }
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            c_compiler: default_c_compiler(),
            cpp_compiler: default_cpp_compiler(),
        }
    }
}

fn default_c_compiler() -> String {
    "clang".to_owned()
}

fn default_cpp_compiler() -> String {
    "clang++".to_owned()
}
