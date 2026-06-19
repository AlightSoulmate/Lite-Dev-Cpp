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

#[cfg(test)]
mod tests {
    use super::{is_c_source, is_cpp_source, is_supported_source};
    use std::path::Path;

    #[test]
    fn recognizes_supported_source_extensions_case_insensitively() {
        assert!(is_c_source(Path::new("main.C")));
        assert!(is_cpp_source(Path::new("main.CPP")));
        assert!(is_cpp_source(Path::new("main.cxx")));
        assert!(!is_supported_source(Path::new("main.hpp")));
        assert!(!is_supported_source(Path::new("README")));
    }
}
