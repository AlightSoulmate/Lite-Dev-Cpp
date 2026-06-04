use std::{
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub fn reveal_in_finder(path: &Path) -> io::Result<()> {
    Command::new("open")
        .arg("-R")
        .arg(path)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

pub fn create_file(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let path = dir.join(name);
    if path.exists() {
        return Err(format!("File already exists: {}", path.display()));
    }
    fs::File::create(&path)
        .map_err(|err| format!("Could not create file {}: {err}", path.display()))?;
    Ok(path)
}

pub fn create_folder(dir: &Path, name: &str) -> Result<PathBuf, String> {
    let path = dir.join(name);
    if path.exists() {
        return Err(format!("Folder already exists: {}", path.display()));
    }
    fs::create_dir(&path)
        .map_err(|err| format!("Could not create folder {}: {err}", path.display()))?;
    Ok(path)
}

pub fn rename_path(path: &Path, new_name: &str) -> Result<PathBuf, String> {
    let parent = path
        .parent()
        .ok_or_else(|| "Cannot rename a path without a parent folder.".to_owned())?;
    let new_path = parent.join(new_name);
    if new_path.exists() {
        return Err(format!("Target already exists: {}", new_path.display()));
    }
    fs::rename(path, &new_path).map_err(|err| {
        format!(
            "Could not rename {} to {}: {err}",
            path.display(),
            new_path.display()
        )
    })?;
    Ok(new_path)
}

pub fn move_to_trash(path: &Path) -> io::Result<()> {
    let script = format!(
        "tell application \"Finder\" to delete POSIX file {}",
        apple_script_string(&path.to_string_lossy())
    );
    let status = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "Finder returned exit code {}",
            status.code().unwrap_or(-1)
        )))
    }
}

pub fn unique_child_name(dir: &Path, preferred: &str) -> String {
    if !dir.join(preferred).exists() {
        return preferred.to_owned();
    }

    let path = Path::new(preferred);
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy())
        .unwrap_or_default();
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy());

    for index in 2..1000 {
        let candidate = match extension.as_ref() {
            Some(extension) => format!("{stem} {index}.{extension}"),
            None => format!("{stem} {index}"),
        };
        if !dir.join(&candidate).exists() {
            return candidate;
        }
    }

    preferred.to_owned()
}

pub fn path_contains(container: &Path, child: &Path) -> bool {
    child == container || child.starts_with(container)
}

fn apple_script_string(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}
