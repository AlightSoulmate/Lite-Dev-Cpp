use std::{
    fs, io,
    path::{Path, PathBuf},
};

// File tree model

#[derive(Debug, Clone)]
pub struct Project {
    root: PathBuf,
    nodes: Vec<FileNode>,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileNode>,
}

impl Project {
    pub fn open(root: PathBuf) -> io::Result<Self> {
        let mut project = Self {
            root,
            nodes: Vec::new(),
        };
        project.refresh()?;
        Ok(project)
    }

    pub fn refresh(&mut self) -> io::Result<()> {
        self.nodes = read_children(&self.root)?;
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn nodes(&self) -> &[FileNode] {
        &self.nodes
    }
}

fn read_children(root: &Path) -> io::Result<Vec<FileNode>> {
    let mut entries = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();

        if should_skip(&name) {
            continue;
        }

        let file_type = entry.file_type()?;
        let is_dir = file_type.is_dir();
        let children = if is_dir {
            read_children(&path).unwrap_or_default()
        } else {
            Vec::new()
        };

        entries.push(FileNode {
            name,
            path,
            is_dir,
            children,
        });
    }

    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(entries)
}

fn should_skip(name: &str) -> bool {
    matches!(name, ".git" | "target" | ".DS_Store")
}

#[cfg(test)]
mod tests {
    use super::should_skip;

    #[test]
    fn skips_internal_project_entries_only() {
        assert!(should_skip(".git"));
        assert!(should_skip("target"));
        assert!(!should_skip("a"));
        assert!(!should_skip("main.cpp"));
    }
}
