use std::path::{Path, PathBuf};

const PORTABLE_MARKER_NAMES: &[&str] = &[
    "portable.ini",
    "Portable.ini",
    "PORTABLE.INI",
    "portable.txt",
    "Portable.txt",
    "PORTABLE.TXT",
];
const PORTABLE_CONFIG_DIR: &str = "AimRust.config";
const DEFAULT_CONFIG_SUBDIR: &str = "AimRust";

/// Finds candidates for the application directory in order of priority:
/// 1. If running inside an AppImage (`$APPIMAGE` environment variable), the directory containing the AppImage.
/// 2. Original working directory (`$OWD` set by AppImage runtime before AppRun changed dirs).
/// 3. Current working directory (`std::env::current_dir()`).
/// 4. Current executable's directory (`std::env::current_exe()`).
pub fn app_dir_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    // 1. If running as an AppImage, check the folder containing the .AppImage file
    if let Ok(appimage_path) = std::env::var("APPIMAGE") {
        let path = PathBuf::from(appimage_path);
        if let Some(parent) = path.parent()
            && !candidates.contains(&parent.to_path_buf())
        {
            candidates.push(parent.to_path_buf());
        }
    }

    // 2. Original working directory from AppImage runtime
    if let Ok(owd) = std::env::var("OWD") {
        let path = PathBuf::from(owd);
        if !candidates.contains(&path) {
            candidates.push(path);
        }
    }

    // 3. Current working directory
    if let Ok(cwd) = std::env::current_dir()
        && !candidates.contains(&cwd)
    {
        candidates.push(cwd);
    }

    // 4. Executable parent directory
    if let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let path = exe_dir.to_path_buf();
        if !candidates.contains(&path) {
            candidates.push(path);
        }
    }

    candidates
}

/// Checks if any portable marker file exists in the given directory (case-insensitive check).
fn find_portable_marker_in_dir(dir: &Path) -> bool {
    // Fast path: direct check of common casings
    for &marker in PORTABLE_MARKER_NAMES {
        if dir.join(marker).is_file() {
            return true;
        }
    }

    // Fallback: directory entry inspection for case-insensitive matching
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            if let Ok(file_type) = entry.file_type()
                && (file_type.is_file() || file_type.is_symlink())
            {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.eq_ignore_ascii_case("portable.ini")
                    || name_str.eq_ignore_ascii_case("portable.txt")
                {
                    return true;
                }
            }
        }
    }

    false
}

/// Checks if portable mode is enabled, returning the target portable root folder where
/// `AimRust.config` should reside if found.
pub fn portable_root_dir() -> Option<PathBuf> {
    for dir in app_dir_candidates() {
        if find_portable_marker_in_dir(&dir) {
            return Some(dir);
        }
    }
    false_if_appimage_dir()
}

/// Helper to ensure if an AppImage folder has `AimRust.config` already present,
/// it can also be treated as portable root even if marker check had an edge case.
fn false_if_appimage_dir() -> Option<PathBuf> {
    for dir in app_dir_candidates() {
        let config_folder = dir.join(PORTABLE_CONFIG_DIR);
        if config_folder.is_dir() {
            return Some(dir);
        }
    }
    None
}

/// Returns true if portable mode is active.
#[allow(dead_code)]
pub fn is_portable() -> bool {
    portable_root_dir().is_some()
}

/// Returns the base directory for storing configuration and runtime data.
/// If `portable.ini` (or `AimRust.config`) is detected, this returns `<root>/AimRust.config`,
/// ensuring the directory exists.
/// Otherwise, falls back to the system config directory `dirs::config_dir()/AimRust` (or the provided `fallback_subdir`).
pub fn base_config_dir() -> Option<PathBuf> {
    base_config_dir_with_fallback(DEFAULT_CONFIG_SUBDIR)
}

/// Returns the base configuration directory, using `fallback_subdir` under `dirs::config_dir()`
/// when not in portable mode.
pub fn base_config_dir_with_fallback(fallback_subdir: &str) -> Option<PathBuf> {
    if let Some(portable_root) = portable_root_dir() {
        let dir = portable_root.join(PORTABLE_CONFIG_DIR);
        let _ = std::fs::create_dir_all(&dir);
        Some(dir)
    } else {
        dirs::config_dir().map(|dir| dir.join(fallback_subdir))
    }
}

/// Resolves a configuration file path given its filename.
pub fn config_file_path(filename: &str) -> Option<PathBuf> {
    base_config_dir().map(|dir| dir.join(filename))
}

/// Resolves a configuration file path with a custom fallback subdirectory for non-portable mode.
pub fn config_file_path_with_fallback(filename: &str, fallback_subdir: &str) -> Option<PathBuf> {
    base_config_dir_with_fallback(fallback_subdir).map(|dir| dir.join(filename))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portable_check() {
        let dir = base_config_dir();
        assert!(dir.is_some());
    }

    #[test]
    fn test_portable_detection_case_insensitive() {
        let temp_dir = std::env::temp_dir().join("aimrust_test_portable_case");
        let _ = std::fs::create_dir_all(&temp_dir);
        let marker = temp_dir.join("Portable.INI");

        assert!(!find_portable_marker_in_dir(&temp_dir));

        std::fs::write(&marker, "").unwrap();
        assert!(find_portable_marker_in_dir(&temp_dir));

        let _ = std::fs::remove_file(marker);
        let _ = std::fs::remove_dir_all(temp_dir);
    }
}
