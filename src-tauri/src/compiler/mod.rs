//! Core Cinema 4D plugin compiler services.

pub mod build;
pub mod env;
pub mod jobs;
pub mod package;
pub mod sdk;

use std::path::PathBuf;

use crate::types::CompilerPlatform;

pub const APP_CACHE_FOLDER: &str = "C4DPluginCompiler";
pub const DEFAULT_PRESET_WINDOWS: &str = "windows_vs2022_v143_x64";
const LEGACY_PRESET_WINDOWS: &str = "windows_vs2022_v143";
const WINDOWS_PRESET_PRIORITY: &[&str] = &[DEFAULT_PRESET_WINDOWS, LEGACY_PRESET_WINDOWS];
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

pub fn select_build_preset<'a>(
    build_platform: &BuildPlatform,
    presets: &'a [String],
) -> Option<&'a str> {
    if matches!(build_platform.platform, CompilerPlatform::Windows) {
        return WINDOWS_PRESET_PRIORITY.iter().find_map(|candidate| {
            presets
                .iter()
                .find(|preset| preset.as_str() == *candidate)
                .map(String::as_str)
        });
    }

    build_platform.preset.and_then(|expected| {
        presets
            .iter()
            .find(|preset| preset.as_str() == expected)
            .map(String::as_str)
    })
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
    use super::{current_build_platform, sanitize_path_name, DEFAULT_PRESET_WINDOWS};

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

    #[test]
    #[cfg(target_os = "windows")]
    fn selects_current_windows_cmake_preset() {
        let build_platform = current_build_platform();
        let sdk_2026_presets = vec![
            "linux_ninja".to_string(),
            DEFAULT_PRESET_WINDOWS.to_string(),
            "windows_vs2022_clangcl_x64".to_string(),
        ];
        assert_eq!(
            super::select_build_preset(&build_platform, &sdk_2026_presets),
            Some(DEFAULT_PRESET_WINDOWS)
        );

        let legacy_presets = vec!["windows_vs2022_v143".to_string()];
        assert_eq!(
            super::select_build_preset(&build_platform, &legacy_presets),
            Some("windows_vs2022_v143")
        );
    }
}
