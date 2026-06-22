//! Cinema 4D 插件构建环境检测。

use std::path::PathBuf;
use std::process::Command;

use crate::compiler::sdk;
use crate::compiler::{current_build_platform, local_data_root};
use crate::types::{
    CompilerPlatform, EnvironmentReport, InstalledC4dVersion, InstalledSdkZip, SdkVersionOption,
    SetupRequirement, SetupRequirementStatus, ToolStatus,
};

pub fn detect_environment() -> EnvironmentReport {
    let build_platform = current_build_platform();
    let supported = build_platform.preset.is_some();
    let cmake = detect_cmake();
    let visual_studio = detect_visual_studio();
    let windows_sdk = detect_windows_sdk();
    let xcode = detect_xcode();
    let clang = detect_clang();
    let python = detect_python();
    let installed_sdk_zips = detect_installed_sdk_zips();
    let installed_c4d_versions = sdk::detect_installed_c4d_versions();
    let cache_root = local_data_root()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|error| error);

    EnvironmentReport {
        os: std::env::consts::OS.to_string(),
        supported,
        compiler_platform: build_platform.platform,
        cmake_preset: build_platform.preset.map(str::to_string),
        binary_extension: build_platform.binary_extension.map(str::to_string),
        cmake,
        visual_studio,
        windows_sdk,
        xcode,
        clang,
        python,
        installed_sdk_zips,
        installed_c4d_versions,
        cache_root,
    }
}

pub fn setup_requirements(
    installed_c4d_versions: &[InstalledC4dVersion],
    sdk_versions: &[SdkVersionOption],
    sdk_root: Option<&str>,
) -> Vec<SetupRequirement> {
    let report = detect_environment();
    let mut requirements = Vec::new();

    requirements.push(c4d_requirement(installed_c4d_versions));
    requirements.push(sdk_root_requirement(sdk_root));
    requirements.extend(sdk_version_requirements(
        installed_c4d_versions,
        sdk_versions,
    ));
    requirements.push(tool_requirement(
        "cmake",
        "CMake 3.30+",
        &report.cmake,
        true,
        cmake_install_hint(),
    ));
    requirements.push(tool_requirement(
        "python",
        "Python",
        &report.python,
        false,
        Some("Install Python 3.10+ and ensure it is available in PATH.".to_string()),
    ));

    match std::env::consts::OS {
        "macos" => {
            requirements.push(tool_requirement(
                "xcode",
                "Xcode 16+",
                &report.xcode,
                false,
                Some(
                    "Install Xcode from the App Store, then run xcode-select if needed."
                        .to_string(),
                ),
            ));
            requirements.push(tool_requirement(
                "clang",
                "Apple Clang",
                &report.clang,
                false,
                Some("Install Xcode command line tools with xcode-select --install.".to_string()),
            ));
        }
        "windows" => {
            requirements.push(tool_requirement(
                "visual_studio",
                "Visual Studio 2022 C++",
                &report.visual_studio,
                false,
                Some("Install Visual Studio 2022 with Desktop development with C++.".to_string()),
            ));
            requirements.push(tool_requirement(
                "windows_sdk",
                "Windows 10/11 SDK",
                &report.windows_sdk,
                false,
                Some("Install the Windows SDK through Visual Studio Installer.".to_string()),
            ));
        }
        _ => {
            requirements.push(SetupRequirement {
                key: "platform".to_string(),
                label: "Build platform".to_string(),
                status: if report.supported {
                    SetupRequirementStatus::Ready
                } else {
                    SetupRequirementStatus::Manual
                },
                detail: if report.supported {
                    format!(
                        "Using {}",
                        compiler_platform_label(&report.compiler_platform)
                    )
                } else {
                    "This operating system is not configured for C4D SDK builds yet.".to_string()
                },
                path: None,
                version: None,
                auto_installable: false,
                install_hint: None,
            });
        }
    }

    requirements
}

fn c4d_requirement(installed_c4d_versions: &[InstalledC4dVersion]) -> SetupRequirement {
    if installed_c4d_versions.is_empty() {
        return SetupRequirement {
            key: "cinema4d".to_string(),
            label: "Cinema 4D 2024.4+".to_string(),
            status: SetupRequirementStatus::Manual,
            detail: "No local Cinema 4D installation was detected.".to_string(),
            path: None,
            version: None,
            auto_installable: false,
            install_hint: Some(
                "Install Cinema 4D 2024.4 or newer before building plugins.".to_string(),
            ),
        };
    }

    let latest = &installed_c4d_versions[0];
    SetupRequirement {
        key: "cinema4d".to_string(),
        label: "Cinema 4D 2024.4+".to_string(),
        status: SetupRequirementStatus::Ready,
        detail: format!("Detected {} installation(s).", installed_c4d_versions.len()),
        path: Some(latest.path.clone()),
        version: Some(latest.version.clone()),
        auto_installable: false,
        install_hint: None,
    }
}

fn sdk_root_requirement(sdk_root: Option<&str>) -> SetupRequirement {
    let Some(root) = sdk_root.filter(|item| !item.trim().is_empty()) else {
        return SetupRequirement {
            key: "sdk_root".to_string(),
            label: "SDK root".to_string(),
            status: SetupRequirementStatus::Missing,
            detail: "Choose a local SDK root folder without spaces.".to_string(),
            path: None,
            version: None,
            auto_installable: true,
            install_hint: Some("Use one-click setup to create Documents/Maxon_SDK.".to_string()),
        };
    };

    let has_spaces = root.chars().any(char::is_whitespace);
    SetupRequirement {
        key: "sdk_root".to_string(),
        label: "SDK root".to_string(),
        status: if has_spaces {
            SetupRequirementStatus::Warning
        } else {
            SetupRequirementStatus::Ready
        },
        detail: if has_spaces {
            "SDK root contains spaces; Maxon CMake/Xcode scripts can fail on this path.".to_string()
        } else {
            "SDK root is configured.".to_string()
        },
        path: Some(root.to_string()),
        version: None,
        auto_installable: true,
        install_hint: Some("Use a path like ~/Documents/Maxon_SDK.".to_string()),
    }
}

fn sdk_version_requirements(
    installed_c4d_versions: &[InstalledC4dVersion],
    sdk_versions: &[SdkVersionOption],
) -> Vec<SetupRequirement> {
    let mut required_versions = installed_c4d_versions
        .iter()
        .map(|item| item.sdk_version.clone())
        .collect::<Vec<_>>();
    required_versions.sort();
    required_versions.dedup();

    if required_versions.is_empty() {
        required_versions.push(sdk::DEFAULT_MIN_SDK_VERSION.to_string());
    }

    required_versions
        .into_iter()
        .map(|version| {
            let option = sdk_versions.iter().find(|item| item.version == version);
            let ready =
                option.is_some_and(|item| item.sdk_root.is_some() || item.sdk_zip.is_some());
            let download_url = option.and_then(|item| item.download_url.clone());
            SetupRequirement {
                key: format!("sdk_{version}"),
                label: format!("C++ SDK {version}"),
                status: if ready {
                    SetupRequirementStatus::Ready
                } else if download_url.is_some() {
                    SetupRequirementStatus::Missing
                } else {
                    SetupRequirementStatus::Manual
                },
                detail: option
                    .map(|item| item.status.clone())
                    .unwrap_or_else(|| "No SDK source is configured.".to_string()),
                path: option
                    .and_then(|item| item.sdk_root.clone().or_else(|| item.sdk_zip.clone())),
                version: Some(version.clone()),
                auto_installable: download_url.is_some(),
                install_hint: download_url
                    .map(|url| format!("One-click setup can download and extract {url}.")),
            }
        })
        .collect()
}

fn tool_requirement(
    key: &str,
    label: &str,
    tool: &ToolStatus,
    auto_installable: bool,
    install_hint: Option<String>,
) -> SetupRequirement {
    SetupRequirement {
        key: key.to_string(),
        label: label.to_string(),
        status: if tool.found {
            SetupRequirementStatus::Ready
        } else if auto_installable {
            SetupRequirementStatus::Missing
        } else {
            SetupRequirementStatus::Manual
        },
        detail: tool.message.clone().unwrap_or_else(|| {
            if tool.found {
                "Detected.".to_string()
            } else {
                "Not detected.".to_string()
            }
        }),
        path: tool.path.clone(),
        version: tool.version.clone(),
        auto_installable,
        install_hint,
    }
}

fn cmake_install_hint() -> Option<String> {
    let hint = match std::env::consts::OS {
        "macos" => "Install CMake with Homebrew: brew install cmake.",
        "windows" => "Install CMake with winget: winget install Kitware.CMake.",
        "linux" => "Install CMake 3.30+ with your system package manager or Kitware packages.",
        _ => "Install CMake 3.30+ and ensure it is available in PATH.",
    };

    Some(hint.to_string())
}

fn compiler_platform_label(platform: &CompilerPlatform) -> &'static str {
    match platform {
        CompilerPlatform::Windows => "Windows",
        CompilerPlatform::Macos => "macOS",
        CompilerPlatform::Linux => "Linux",
        CompilerPlatform::Unsupported => "unsupported platform",
    }
}

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "macos")]
pub fn detect_installed_sdk_zips() -> Vec<InstalledSdkZip> {
    let mut zips = Vec::new();
    for version in ["2024", "2025", "2026"] {
        let path = PathBuf::from(format!("/Applications/Maxon Cinema 4D {version}/sdk.zip"));
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

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub fn detect_installed_sdk_zips() -> Vec<InstalledSdkZip> {
    Vec::new()
}

#[cfg(target_os = "windows")]
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

#[cfg(not(target_os = "windows"))]
pub fn detect_cmake_path() -> Option<String> {
    find_program("cmake")
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

#[cfg(target_os = "windows")]
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

#[cfg(not(target_os = "windows"))]
fn detect_visual_studio() -> ToolStatus {
    ToolStatus {
        found: false,
        path: None,
        version: None,
        message: Some("Visual Studio is only required for Windows builds".to_string()),
    }
}

#[cfg(target_os = "windows")]
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

#[cfg(not(target_os = "windows"))]
fn detect_windows_sdk() -> ToolStatus {
    ToolStatus {
        found: false,
        path: None,
        version: None,
        message: Some("Windows SDK is only required for Windows builds".to_string()),
    }
}

#[cfg(target_os = "macos")]
fn detect_xcode() -> ToolStatus {
    let version = run_capture("xcodebuild", &["-version"]).ok();
    ToolStatus {
        found: version.is_some(),
        path: find_program("xcodebuild"),
        version: version
            .as_deref()
            .and_then(|text| text.lines().next().map(str::to_string)),
        message: if version.is_some() {
            Some("Xcode command line tools detected".to_string())
        } else {
            Some("Xcode was not found; install Xcode 16 or newer".to_string())
        },
    }
}

#[cfg(not(target_os = "macos"))]
fn detect_xcode() -> ToolStatus {
    ToolStatus {
        found: false,
        path: None,
        version: None,
        message: Some("Xcode is only required for macOS builds".to_string()),
    }
}

#[cfg(target_os = "macos")]
fn detect_clang() -> ToolStatus {
    let path = run_capture("xcrun", &["--find", "clang"])
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty());
    let version = run_capture("xcrun", &["clang", "--version"])
        .ok()
        .and_then(|text| text.lines().next().map(str::to_string));

    ToolStatus {
        found: path.is_some(),
        path,
        version,
        message: None,
    }
}

#[cfg(not(target_os = "macos"))]
fn detect_clang() -> ToolStatus {
    ToolStatus {
        found: false,
        path: None,
        version: None,
        message: Some("Clang through Xcode is only required for macOS builds".to_string()),
    }
}

fn detect_python() -> ToolStatus {
    let python = find_program("python")
        .or_else(|| find_program("python3"))
        .unwrap_or_default();
    if python.is_empty() {
        return ToolStatus {
            found: false,
            path: None,
            version: None,
            message: Some("Python was not found in PATH".to_string()),
        };
    }

    let version = run_capture(&python, &["--version"])
        .ok()
        .and_then(|text| text.lines().next().map(str::to_string));

    ToolStatus {
        found: true,
        path: Some(python),
        version,
        message: None,
    }
}

#[cfg(target_os = "windows")]
fn find_program(program: &str) -> Option<String> {
    run_capture("where", &[program])
        .ok()
        .and_then(|text| text.lines().next().map(|line| line.trim().to_string()))
        .filter(|line| !line.is_empty())
}

#[cfg(not(target_os = "windows"))]
fn find_program(program: &str) -> Option<String> {
    run_capture("which", &[program])
        .ok()
        .map(|text| text.trim().to_string())
        .filter(|text| !text.is_empty())
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
