use std::{fs, io, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

// App and compiler config

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    pub const FILE_NAME: &'static str = "config.toml";

    pub fn load() -> io::Result<Self> {
        let path = Self::config_path()?;
        if !path.exists() {
            return Ok(Self::default());
        }

        let contents = fs::read_to_string(path)?;
        toml::from_str(&contents).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid app config: {err}"),
            )
        })
    }

    pub fn save(&self) -> io::Result<PathBuf> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("could not serialize config: {err}"),
            )
        })?;
        fs::write(&path, contents)?;
        Ok(path)
    }

    pub fn config_path() -> io::Result<PathBuf> {
        ProjectDirs::from("dev", "LiteDevCpp", "Lite-Dev-Cpp")
            .map(|dirs| dirs.config_dir().join(Self::FILE_NAME))
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "could not find the user config directory",
                )
            })
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
