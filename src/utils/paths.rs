use std::path::Path;

pub fn is_supported_source(path: &Path) -> bool {
    is_c_source(path) || is_cpp_source(path)
}

pub fn is_c_source(path: &Path) -> bool {
    extension_eq(path, "c")
}

pub fn is_cpp_source(path: &Path) -> bool {
    matches!(extension_lower(path).as_deref(), Some("cpp" | "cc" | "cxx"))
}

fn extension_eq(path: &Path, expected: &str) -> bool {
    extension_lower(path).is_some_and(|ext| ext == expected)
}

fn extension_lower(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
}
