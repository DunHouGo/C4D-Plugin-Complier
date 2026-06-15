//! Environment detection for Cinema 4D plugin builds.

use std::path::PathBuf;
use std::process::Command;

use crate::compiler::local_data_root;
use crate::compiler::sdk;
use crate::types::{EnvironmentReport, InstalledSdkZip, ToolStatus};

pub fn detect_environment() -> EnvironmentReport {
    let supported = cfg!(target_os = "windows");
    let cmake = detect_cmake();
    let visual_studio = detect_visual_studio();
    let windows_sdk = detect_windows_sdk();
    let installed_sdk_zips = detect_installed_sdk_zips();
    let installed_c4d_versions = sdk::detect_installed_c4d_versions();
    let cache_root = local_data_root()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|error| error);

    EnvironmentReport {
        os: std::env::consts::OS.to_string(),
        supported,
        cmake,
        visual_studio,
        windows_sdk,
        installed_sdk_zips,
        installed_c4d_versions,
        cache_root,
    }
}

pub fn detect_installed_sdk_zips() -> Vec<InstalledSdkZip> {
    let mut zips = Vec::new();
    for version in ["2024", "2025", "2026"] {
        let path = PathBuf::from(format!(
            r"C:\Program Files\Maxon Cinema 4D {version}\sdk.zip"
        ));
        if let Ok(metadata) = std::fs::metadata(&path) {
            zips.push(InstalledSdkZip {
                version: version.to_string(),
                path: path.display().to_string(),
                size_bytes: metadata.len() as f64,
            });
        }
    }
    zips
}

pub fn detect_cmake_path() -> Option<String> {
    if let Some(path) = run_capture("where", &["cmake"])
        .ok()
        .and_then(|text| text.lines().next().map(|line| line.trim().to_string()))
        .filter(|line| !line.is_empty())
    {
        return Some(path);
    }

    let common = PathBuf::from(r"C:\Program Files\CMake\bin\cmake.exe");
    if common.is_file() {
        return Some(common.display().to_string());
    }

    None
}

fn detect_cmake() -> ToolStatus {
    let Some(path) = detect_cmake_path() else {
        return ToolStatus {
            found: false,
            path: None,
            version: None,
            message: Some("CMake was not found in PATH or the default install path".to_string()),
        };
    };

    let version = run_capture(&path, &["--version"])
        .ok()
        .and_then(|text| text.lines().next().map(|line| line.to_string()));

    ToolStatus {
        found: true,
        path: Some(path),
        version,
        message: None,
    }
}

fn detect_visual_studio() -> ToolStatus {
    let vswhere =
        PathBuf::from(r"C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe");
    if !vswhere.is_file() {
        return ToolStatus {
            found: false,
            path: None,
            version: None,
            message: Some("vswhere.exe was not found".to_string()),
        };
    }

    let args = [
        "-latest",
        "-products",
        "*",
        "-requires",
        "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
        "-property",
        "installationPath",
    ];
    match run_capture(&vswhere.display().to_string(), &args) {
        Ok(output) => {
            let path = output.trim().to_string();
            ToolStatus {
                found: !path.is_empty(),
                path: if path.is_empty() { None } else { Some(path) },
                version: None,
                message: if output.trim().is_empty() {
                    Some("Visual Studio 2022 with VC Tools was not found".to_string())
                } else {
                    Some("Visual Studio with VC Tools detected".to_string())
                },
            }
        }
        Err(error) => ToolStatus {
            found: false,
            path: None,
            version: None,
            message: Some(error),
        },
    }
}

fn detect_windows_sdk() -> ToolStatus {
    let sdk_root = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\Include");
    if !sdk_root.is_dir() {
        return ToolStatus {
            found: false,
            path: None,
            version: None,
            message: Some("Windows 10 SDK include folder was not found".to_string()),
        };
    }

    let latest = std::fs::read_dir(&sdk_root)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .filter_map(|entry| {
            entry
                .file_type()
                .ok()
                .filter(|kind| kind.is_dir())
                .map(|_| entry.file_name().to_string_lossy().to_string())
        })
        .max();

    ToolStatus {
        found: latest.is_some(),
        path: Some(sdk_root.display().to_string()),
        version: latest,
        message: None,
    }
}

pub fn run_capture(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("Failed to run {program}: {error}"))?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
