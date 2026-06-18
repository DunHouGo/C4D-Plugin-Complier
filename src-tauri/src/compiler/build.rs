//! CMake build orchestration for Cinema 4D plugins.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use walkdir::WalkDir;

use crate::compiler::env::detect_cmake_path;
use crate::compiler::package::package_outputs;
use crate::compiler::sdk::{prepare_sdk, read_configure_presets};
use crate::compiler::{
    current_build_platform, local_data_root, parse_version_list, require_dir, sanitize_path_name,
};
use crate::types::{BuildArtifact, BuildLogEvent, BuildProgressEvent, BuildRequest};

pub type LogCallback<'a> = &'a dyn Fn(BuildLogEvent);
pub type ProgressCallback<'a> = &'a dyn Fn(BuildProgressEvent);

pub fn execute_build(
    job_id: &str,
    request: &BuildRequest,
    log: LogCallback<'_>,
    progress: ProgressCallback<'_>,
) -> Result<Vec<BuildArtifact>, String> {
    let build_platform = current_build_platform();
    let preset = build_platform
        .preset
        .ok_or_else(|| "C4D builds are currently supported on Windows and macOS".to_string())?;
    let binary_extension = build_platform
        .binary_extension
        .ok_or_else(|| "No binary extension is configured for this platform".to_string())?;

    validate_request(request)?;
    let cmake = detect_cmake_path().ok_or_else(|| "CMake was not found".to_string())?;
    let versions: Vec<String> = request
        .versions
        .iter()
        .flat_map(|version| parse_version_list(version))
        .collect();
    let total = (versions.len() * request.configuration.cmake_configs().len()) as u32;
    let mut current = 0;
    let mut built_binaries = Vec::new();

    for version in &versions {
        log_info(log, job_id, &format!("Preparing Cinema 4D {version} SDK"));
        let sdk = prepare_sdk(version, request.refresh_sdk_cache)?;
        let sdk_root = PathBuf::from(
            sdk.sdk_root
                .clone()
                .ok_or_else(|| format!("SDK {version} did not resolve to a root path"))?,
        );
        let presets = read_configure_presets(&sdk_root)?;
        if !presets.iter().any(|name| name == preset) {
            return Err(format!(
                "SDK {version} does not provide preset {preset}. Available presets: {}",
                presets.join(", ")
            ));
        }

        let modules_dir = prepare_module_alias(request)?;
        let build_dir = build_dir_for(&sdk_root, version, preset)?;
        configure_sdk(
            &cmake,
            &sdk_root,
            &build_dir,
            &modules_dir,
            preset,
            log,
            job_id,
        )?;

        for configuration in request.configuration.cmake_configs() {
            current += 1;
            progress(BuildProgressEvent {
                job_id: job_id.to_string(),
                current,
                total,
                label: format!("Building C4D {version} {configuration}"),
            });
            build_target(
                &cmake,
                &build_dir,
                configuration,
                &request.module_name,
                log,
                job_id,
            )?;
            let binary = find_plugin_binary(
                &build_dir,
                configuration,
                &request.module_name,
                binary_extension,
            )?;
            built_binaries.push((version.clone(), configuration.to_string(), binary));
        }
    }

    log_info(log, job_id, "Packaging build outputs");
    package_outputs(request, &built_binaries)
}

fn validate_request(request: &BuildRequest) -> Result<(), String> {
    if request.plugin_root.trim().is_empty() {
        return Err("Plugin root is required".to_string());
    }
    if request.module_name.trim().is_empty() {
        return Err("Module name is required".to_string());
    }
    if request.package_name.trim().is_empty() {
        return Err("Package name is required".to_string());
    }
    if request.versions.is_empty() {
        return Err("At least one C4D version is required".to_string());
    }
    require_dir(Path::new(&request.plugin_root))?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn prepare_module_alias(request: &BuildRequest) -> Result<PathBuf, String> {
    let modules_root = local_data_root()?.join("modules").join(format!(
        "{}_modules",
        sanitize_path_name(&request.module_name)
    ));
    std::fs::create_dir_all(&modules_root)
        .map_err(|error| format!("Failed to create {}: {error}", modules_root.display()))?;

    let link = modules_root.join(&request.module_name);
    let target = PathBuf::from(&request.plugin_root)
        .canonicalize()
        .map_err(|error| format!("Failed to resolve plugin root: {error}"))?;

    if link.exists() {
        if same_path(&link, &target) {
            return Ok(modules_root);
        }
        remove_junction(&link)?;
    }

    let status = Command::new("cmd")
        .args(["/c", "mklink", "/J"])
        .arg(&link)
        .arg(&target)
        .status()
        .map_err(|error| format!("Failed to create module junction: {error}"))?;
    if !status.success() {
        return Err(format!(
            "Failed to create module junction: {}",
            link.display()
        ));
    }

    Ok(modules_root)
}

#[cfg(target_os = "macos")]
fn prepare_module_alias(request: &BuildRequest) -> Result<PathBuf, String> {
    let modules_root = local_data_root()?.join("modules").join(format!(
        "{}_modules",
        sanitize_path_name(&request.module_name)
    ));
    std::fs::create_dir_all(&modules_root)
        .map_err(|error| format!("Failed to create {}: {error}", modules_root.display()))?;

    let link = modules_root.join(&request.module_name);
    let target = PathBuf::from(&request.plugin_root)
        .canonicalize()
        .map_err(|error| format!("Failed to resolve plugin root: {error}"))?;

    if link.exists() {
        if same_path(&link, &target) {
            return Ok(modules_root);
        }
        remove_link_or_dir(&link)?;
    }

    std::os::unix::fs::symlink(&target, &link).map_err(|error| {
        format!(
            "Failed to create module symlink {} -> {}: {error}",
            link.display(),
            target.display()
        )
    })?;

    Ok(modules_root)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn prepare_module_alias(_request: &BuildRequest) -> Result<PathBuf, String> {
    Err("C4D builds are currently supported on Windows and macOS".to_string())
}

fn same_path(left: &Path, right: &Path) -> bool {
    left.canonicalize()
        .ok()
        .is_some_and(|left_path| left_path == right)
}

#[cfg(target_os = "windows")]
fn remove_junction(path: &Path) -> Result<(), String> {
    let status = Command::new("cmd")
        .args(["/c", "rmdir"])
        .arg(path)
        .status()
        .map_err(|error| format!("Failed to remove {}: {error}", path.display()))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("Failed to remove {}", path.display()))
    }
}

#[cfg(target_os = "macos")]
fn remove_link_or_dir(path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("Failed to inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        std::fs::remove_file(path)
            .map_err(|error| format!("Failed to remove {}: {error}", path.display()))
    } else {
        std::fs::remove_dir_all(path)
            .map_err(|error| format!("Failed to remove {}: {error}", path.display()))
    }
}

fn build_dir_for(sdk_root: &Path, version: &str, preset: &str) -> Result<PathBuf, String> {
    Ok(local_data_root()?
        .join("builds")
        .join(format!("Cinema4D_{version}"))
        .join(sanitize_path_name(&sdk_root.display().to_string()))
        .join(preset))
}

fn configure_sdk(
    cmake: &str,
    sdk_root: &Path,
    build_dir: &Path,
    modules_dir: &Path,
    preset: &str,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<(), String> {
    std::fs::create_dir_all(build_dir)
        .map_err(|error| format!("Failed to create {}: {error}", build_dir.display()))?;
    let args = vec![
        "--preset".to_string(),
        preset.to_string(),
        "-B".to_string(),
        build_dir.display().to_string(),
        format!("-DMAXON_SDK_MODULES_DIR={}", modules_dir.display()),
        "-DMAXON_SDK_CUSTOM_PATHS_FILE=".to_string(),
    ];
    run_command(cmake, &args, sdk_root, log, job_id)
}

fn build_target(
    cmake: &str,
    build_dir: &Path,
    configuration: &str,
    target: &str,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<(), String> {
    let args = vec![
        "--build".to_string(),
        build_dir.display().to_string(),
        "--config".to_string(),
        configuration.to_string(),
        "--target".to_string(),
        target.to_string(),
    ];
    run_command(cmake, &args, build_dir, log, job_id)
}

fn run_command(
    program: &str,
    args: &[String],
    cwd: &Path,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<(), String> {
    log_info(
        log,
        job_id,
        &format!("Running: {program} {}", args.join(" ")),
    );
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("Failed to run {program}: {error}"))?;

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        log_info(log, job_id, line);
    }
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        log_warn(log, job_id, line);
    }

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Command failed with status {}", output.status))
    }
}

fn find_plugin_binary(
    build_dir: &Path,
    configuration: &str,
    module_name: &str,
    binary_extension: &str,
) -> Result<PathBuf, String> {
    let plugin_dir = build_dir
        .join("bin")
        .join(configuration)
        .join("plugins")
        .join(module_name);
    let expected = plugin_dir.join(format!("{module_name}.{binary_extension}"));
    if expected.is_file() {
        return Ok(expected);
    }

    let expected_file_name = format!("{module_name}.{binary_extension}");
    WalkDir::new(build_dir)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_path_buf())
        .find(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|file_name| file_name == expected_file_name)
        })
        .ok_or_else(|| {
            format!(
                "Built binary {expected_file_name} was not found under {}",
                build_dir.display()
            )
        })
}

fn log_info(log: LogCallback<'_>, job_id: &str, message: &str) {
    log(BuildLogEvent {
        job_id: job_id.to_string(),
        level: "info".to_string(),
        message: message.to_string(),
    });
}

fn log_warn(log: LogCallback<'_>, job_id: &str, message: &str) {
    log(BuildLogEvent {
        job_id: job_id.to_string(),
        level: "warn".to_string(),
        message: message.to_string(),
    });
}
