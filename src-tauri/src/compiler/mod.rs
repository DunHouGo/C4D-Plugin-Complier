//! Core Cinema 4D plugin compiler services.

pub mod build;
pub mod env;
pub mod jobs;
pub mod package;
pub mod sdk;

use std::path::PathBuf;

use crate::types::CompilerPlatform;

pub const APP_CACHE_FOLDER: &str = "C4DPluginCompiler";
pub const DEFAULT_PRESET_WINDOWS: &str = "windows_vs2022_v143";
pub const DEFAULT_PRESET_MACOS: &str = "macos_universal_xcode";

pub struct BuildPlatform {
    pub platform: CompilerPlatform,
    pub preset: Option<&'static str>,
    pub binary_extension: Option<&'static str>,
}

pub fn current_build_platform() -> BuildPlatform {
    if cfg!(target_os = "windows") {
        return BuildPlatform {
            platform: CompilerPlatform::Windows,
            preset: Some(DEFAULT_PRESET_WINDOWS),
            binary_extension: Some("xdl64"),
        };
    }

    if cfg!(target_os = "macos") {
        return BuildPlatform {
            platform: CompilerPlatform::Macos,
            preset: Some(DEFAULT_PRESET_MACOS),
            binary_extension: Some("xlib"),
        };
    }

    if cfg!(target_os = "linux") {
        return BuildPlatform {
            platform: CompilerPlatform::Linux,
            preset: None,
            binary_extension: Some("xso64"),
        };
    }

    BuildPlatform {
        platform: CompilerPlatform::Unsupported,
        preset: None,
        binary_extension: None,
    }
}

pub fn local_data_root() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|path| path.join(".boghma").join(APP_CACHE_FOLDER))
        .ok_or_else(|| "Failed to resolve home directory".to_string())
}

pub fn sanitize_path_name(value: &str) -> String {
    let mut output = String::new();
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '.') {
            output.push(ch);
        } else {
            output.push('_');
        }
    }
    output.trim_matches('_').to_string()
}

pub fn parse_version_list(value: &str) -> Vec<String> {
    if let Some((start, end)) = value.split_once('-') {
        if let (Ok(start), Ok(end)) = (start.trim().parse::<u32>(), end.trim().parse::<u32>()) {
            if start <= end {
                return (start..=end).map(|version| version.to_string()).collect();
            }
        }
    }

    value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(str::to_string)
        .collect()
}

pub fn require_dir(path: &std::path::Path) -> Result<(), String> {
    if path.is_dir() {
        Ok(())
    } else {
        Err(format!("Required directory not found: {}", path.display()))
    }
}

pub fn require_file(path: &std::path::Path) -> Result<(), String> {
    if path.is_file() {
        Ok(())
    } else {
        Err(format!("Required file not found: {}", path.display()))
    }
}

#[cfg(test)]
mod tests {
    use super::sanitize_path_name;

    #[test]
    fn sanitizes_path_names() {
        assert_eq!(sanitize_path_name("Cinema 4D 2026"), "Cinema_4D_2026");
        assert_eq!(sanitize_path_name("a/b:c"), "a_b_c");
    }

    #[test]
    fn parses_version_ranges() {
        assert_eq!(
            super::parse_version_list("2024-2026"),
            ["2024", "2025", "2026"]
        );
        assert_eq!(super::parse_version_list("2025, 2026"), ["2025", "2026"]);
    }
}
