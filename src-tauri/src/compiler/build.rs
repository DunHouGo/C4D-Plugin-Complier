//! CMake build orchestration for Cinema 4D plugins.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
use serde::Deserialize;
use walkdir::WalkDir;

use crate::compiler::env::detect_cmake_path;
use crate::compiler::package::package_outputs;
use crate::compiler::sdk::{
    is_cmake_sdk_root, is_legacy_sdk_root, prepare_sdk, read_configure_presets,
};
use crate::compiler::{
    current_build_platform, local_data_root, parse_version_list, require_dir, sanitize_path_name,
    select_build_preset,
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
    if build_platform.preset.is_none() {
        return Err("C4D builds are currently supported on Windows and macOS".to_string());
    }
    let binary_extension = build_platform
        .binary_extension
        .ok_or_else(|| "No binary extension is configured for this platform".to_string())?;

    validate_request(request)?;
    let versions: Vec<String> = request
        .versions
        .iter()
        .flat_map(|version| parse_version_list(version))
        .collect();
    let total = (versions.len() * request.configuration.cmake_configs().len()) as u32;
    let mut current = 0;
    let mut built_binaries = Vec::new();

    for version in &versions {
        log_sdk(log, job_id, &format!("Preparing Cinema 4D {version} SDK"));
        let sdk = prepare_sdk(version, request.refresh_sdk_cache)?;
        let sdk_root = PathBuf::from(
            sdk.sdk_root
                .clone()
                .ok_or_else(|| format!("SDK {version} did not resolve to a root path"))?,
        );

        if is_cmake_sdk_root(&sdk_root) {
            let cmake = detect_cmake_path().ok_or_else(|| "CMake was not found".to_string())?;
            let presets = read_configure_presets(&sdk_root)?;
            let preset = select_build_preset(&build_platform, &presets).ok_or_else(|| {
                let expected = build_platform
                    .preset
                    .unwrap_or("a supported platform preset");
                format!(
                    "SDK {version} does not provide preset {expected}. Available presets: {}",
                    presets.join(", ")
                )
            })?;
            log_sdk(log, job_id, &format!("Using CMake preset '{preset}'"));

            let module_alias_name = cmake_target_name(&request.module_name);
            let build_module_name = resolve_cmake_build_module_name(
                Path::new(&request.plugin_root),
                &module_alias_name,
                log,
                job_id,
            )?;
            let module_alias = prepare_module_alias(request, &module_alias_name)?;
            let build_dir = build_dir_for(&sdk_root, version, preset)?;
            if request.clean_output && build_dir.exists() {
                log_sdk(
                    log,
                    job_id,
                    &format!("Cleaning SDK build directory: {}", build_dir.display()),
                );
                std::fs::remove_dir_all(&build_dir)
                    .map_err(|error| format!("Failed to clean {}: {error}", build_dir.display()))?;
            }
            configure_sdk(
                &cmake,
                &sdk_root,
                &build_dir,
                &module_alias.modules_dir,
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
                    &build_module_name,
                    log,
                    job_id,
                )?;
                let binary = find_plugin_binary(
                    &build_dir,
                    configuration,
                    &build_module_name,
                    binary_extension,
                )?;
                built_binaries.push((version.clone(), configuration.to_string(), binary));
            }
        } else if is_legacy_sdk_root(&sdk_root) {
            prepare_legacy_module_workspace(&sdk_root, request, log, job_id)?;
            generate_legacy_projects(&sdk_root, log, job_id)?;

            for configuration in request.configuration.cmake_configs() {
                current += 1;
                progress(BuildProgressEvent {
                    job_id: job_id.to_string(),
                    current,
                    total,
                    label: format!("Building C4D {version} {configuration}"),
                });
                let legacy_binary_name = build_legacy_target_with_retry(
                    &sdk_root,
                    configuration,
                    &request.module_name,
                    log,
                    job_id,
                )?;
                let binary = find_legacy_plugin_binary(
                    &sdk_root,
                    configuration,
                    &legacy_binary_name,
                    binary_extension,
                )?;
                built_binaries.push((version.clone(), configuration.to_string(), binary));
            }
        } else {
            return Err(format!("Unsupported SDK layout: {}", sdk_root.display()));
        }
    }

    log_package(log, job_id, "Packaging build outputs");
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

struct ModuleAlias {
    modules_dir: PathBuf,
}

#[cfg(target_os = "windows")]
fn prepare_module_alias(request: &BuildRequest, alias_name: &str) -> Result<ModuleAlias, String> {
    let modules_root = local_data_root()?
        .join("plugin-links")
        .join(format!("{}_modules", sanitize_path_name(alias_name)));
    std::fs::create_dir_all(&modules_root)
        .map_err(|error| format!("Failed to create {}: {error}", modules_root.display()))?;
    remove_stale_module_aliases(&modules_root, alias_name)?;

    let link = modules_root.join(alias_name);
    let target = PathBuf::from(&request.plugin_root)
        .canonicalize()
        .map_err(|error| format!("Failed to resolve plugin root: {error}"))?;

    if link.exists() {
        if same_path(&link, &target) {
            return Ok(ModuleAlias {
                modules_dir: modules_root,
            });
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

    Ok(ModuleAlias {
        modules_dir: modules_root,
    })
}

#[cfg(target_os = "macos")]
fn prepare_module_alias(request: &BuildRequest, alias_name: &str) -> Result<ModuleAlias, String> {
    let modules_root = local_data_root()?
        .join("plugin-links")
        .join(format!("{}_modules", sanitize_path_name(alias_name)));
    std::fs::create_dir_all(&modules_root)
        .map_err(|error| format!("Failed to create {}: {error}", modules_root.display()))?;
    remove_stale_module_aliases(&modules_root, alias_name)?;

    let link = modules_root.join(alias_name);
    let target = PathBuf::from(&request.plugin_root)
        .canonicalize()
        .map_err(|error| format!("Failed to resolve plugin root: {error}"))?;

    if link.exists() {
        if same_path(&link, &target) {
            return Ok(ModuleAlias {
                modules_dir: modules_root,
            });
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

    Ok(ModuleAlias {
        modules_dir: modules_root,
    })
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn prepare_module_alias(_request: &BuildRequest, _alias_name: &str) -> Result<ModuleAlias, String> {
    Err("C4D builds are currently supported on Windows and macOS".to_string())
}

fn same_path(left: &Path, right: &Path) -> bool {
    left.canonicalize()
        .ok()
        .is_some_and(|left_path| left_path == right)
}

fn remove_stale_module_aliases(modules_root: &Path, alias_name: &str) -> Result<(), String> {
    for entry in std::fs::read_dir(modules_root)
        .map_err(|error| format!("Failed to read {}: {error}", modules_root.display()))?
    {
        let entry =
            entry.map_err(|error| format!("Failed to read {}: {error}", modules_root.display()))?;
        if entry.file_name().to_string_lossy() == alias_name {
            continue;
        }
        remove_module_alias_entry(&entry.path())?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn remove_module_alias_entry(path: &Path) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path)
        .map_err(|error| format!("Failed to inspect {}: {error}", path.display()))?;
    if metadata.file_type().is_symlink() || metadata.is_file() {
        std::fs::remove_file(path)
            .map_err(|error| format!("Failed to remove {}: {error}", path.display()))
    } else {
        remove_junction(path)
    }
}

#[cfg(target_os = "macos")]
fn remove_module_alias_entry(path: &Path) -> Result<(), String> {
    remove_link_or_dir(path)
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn remove_module_alias_entry(path: &Path) -> Result<(), String> {
    let _ = path;
    Err("C4D builds are currently supported on Windows and macOS".to_string())
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

fn cmake_target_name(module_name: &str) -> String {
    let sanitized = sanitize_path_name(module_name);
    if sanitized.is_empty() {
        module_name.to_string()
    } else {
        sanitized
    }
}

fn resolve_cmake_build_module_name(
    plugin_root: &Path,
    requested_module_name: &str,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<String, String> {
    let direct_project = plugin_root.join("project").join("projectdefinition.txt");
    let direct_cmake_project = plugin_root.join("project").join("CMakeLists.txt");
    if direct_project.is_file() || direct_cmake_project.is_file() {
        return Ok(requested_module_name.to_string());
    }

    let candidates = discover_nested_cmake_module_names(plugin_root);
    if candidates.is_empty() {
        return Ok(requested_module_name.to_string());
    }

    if let Some(candidate) = candidates.iter().find(|candidate| {
        candidate.eq_ignore_ascii_case(requested_module_name)
            || cmake_target_name(candidate).eq_ignore_ascii_case(requested_module_name)
    }) {
        return Ok(candidate.clone());
    }

    if candidates.len() == 1 {
        let module_name = candidates[0].clone();
        log_warn(
            log,
            job_id,
            "cmake",
            &format!(
                "Using nested CMake module target '{module_name}' inside '{}'",
                plugin_root.display()
            ),
        );
        return Ok(module_name);
    }

    Err(format!(
        "Multiple CMake modules were found under {}: {}. Rename the package to match one module or build one module folder at a time.",
        plugin_root.display(),
        candidates.join(", ")
    ))
}

fn discover_nested_cmake_module_names(plugin_root: &Path) -> Vec<String> {
    let mut candidates = WalkDir::new(plugin_root)
        .min_depth(2)
        .max_depth(4)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|file_name| {
                    file_name == "projectdefinition.txt" || file_name == "CMakeLists.txt"
                })
        })
        .filter_map(|entry| c4d_module_name_from_project_file(plugin_root, entry.path()))
        .collect::<Vec<_>>();

    candidates.sort();
    candidates.dedup();
    candidates
}

fn c4d_module_name_from_project_file(plugin_root: &Path, project_file: &Path) -> Option<String> {
    let project_dir = project_file.parent()?;
    if project_dir.file_name().and_then(|value| value.to_str()) != Some("project") {
        return None;
    }
    let module_dir = project_dir.parent()?;
    if module_dir == plugin_root {
        return None;
    }
    module_dir
        .file_name()
        .and_then(|value| value.to_str())
        .map(ToString::to_string)
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

fn prepare_legacy_module_workspace(
    sdk_root: &Path,
    request: &BuildRequest,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<(), String> {
    let module_root = sdk_root.join("plugins").join(&request.module_name);
    if module_root.exists() {
        remove_link_or_dir_cross_platform(&module_root)?;
    }
    copy_dir_recursive(Path::new(&request.plugin_root), &module_root)?;
    ensure_legacy_solution_includes_module(sdk_root, &request.module_name)?;
    log_sdk(
        log,
        job_id,
        &format!(
            "Prepared legacy SDK module workspace: {}",
            module_root.display()
        ),
    );
    Ok(())
}

fn ensure_legacy_solution_includes_module(
    sdk_root: &Path,
    module_name: &str,
) -> Result<(), String> {
    let definition = sdk_root
        .join("plugins")
        .join("project")
        .join("projectdefinition.txt");
    let module_entry = format!("plugins/{module_name}");
    let text = std::fs::read_to_string(&definition)
        .map_err(|error| format!("Failed to read {}: {error}", definition.display()))?;
    if text.contains(&module_entry) {
        return Ok(());
    }

    let mut lines = text.lines().map(str::to_string).collect::<Vec<_>>();
    if let Some(index) = lines
        .iter()
        .rposition(|line| line.trim_start().starts_with("plugins/"))
    {
        let previous = lines[index]
            .trim_end()
            .trim_end_matches('\\')
            .trim_end_matches(';');
        lines[index] = format!("{previous};\\");
        lines.insert(index + 1, format!("\t{module_entry}"));
    } else {
        lines.push("Solution=\\".to_string());
        lines.push(format!("\t{module_entry}"));
    }

    std::fs::write(&definition, format!("{}\n", lines.join("\n")))
        .map_err(|error| format!("Failed to write {}: {error}", definition.display()))
}

fn generate_legacy_projects(
    sdk_root: &Path,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let script = sdk_root.join("generate_solution_osx.command");
        let args = Vec::<String>::new();
        run_command(&script.display().to_string(), &args, sdk_root, log, job_id)
    }

    #[cfg(target_os = "windows")]
    {
        let script = sdk_root.join("generate_solution_win.bat");
        let args = Vec::<String>::new();
        run_command(&script.display().to_string(), &args, sdk_root, log, job_id)
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (sdk_root, log, job_id);
        Err("Legacy SDK builds are currently supported on Windows and macOS".to_string())
    }
}

fn build_legacy_target_with_retry(
    sdk_root: &Path,
    configuration: &str,
    module_name: &str,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<String, String> {
    match build_legacy_target(sdk_root, configuration, module_name, log, job_id) {
        Ok(binary_name) => Ok(binary_name),
        Err(error) if is_missing_legacy_generated_file_error(&error) => {
            log_warn(
                log,
                job_id,
                "toolchain",
                "Legacy SDK generated files were created late; retrying the Xcode build once",
            );
            build_legacy_target(sdk_root, configuration, module_name, log, job_id)
        }
        Err(error) => Err(error),
    }
}

fn build_legacy_target(
    sdk_root: &Path,
    configuration: &str,
    module_name: &str,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        let project = resolve_legacy_xcode_project(sdk_root, module_name, log, job_id)?;
        let scheme = resolve_legacy_xcode_scheme(&project, module_name, log, job_id)?;
        let shim_dir = prepare_python_shim()?;
        let args = vec![
            "-project".to_string(),
            project.display().to_string(),
            "-scheme".to_string(),
            scheme.clone(),
            "-configuration".to_string(),
            configuration.to_string(),
            "-destination".to_string(),
            "generic/platform=macOS".to_string(),
            "-jobs".to_string(),
            "1".to_string(),
            "WARNING_CFLAGS=-Wno-missing-template-arg-list-after-template-kw -Wno-error=overriding-deployment-version".to_string(),
            "build".to_string(),
        ];
        run_command_with_path_prefix(
            "xcodebuild",
            &args,
            project.parent().unwrap_or(sdk_root),
            &shim_dir,
            log,
            job_id,
        )?;
        Ok(scheme)
    }

    #[cfg(target_os = "windows")]
    {
        let _ = (sdk_root, configuration, module_name, log, job_id);
        Err("Legacy Windows SDK build support is not implemented yet".to_string())
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = (sdk_root, configuration, module_name, log, job_id);
        Err("Legacy SDK builds are currently supported on Windows and macOS".to_string())
    }
}

#[cfg(target_os = "macos")]
fn resolve_legacy_xcode_project(
    sdk_root: &Path,
    module_name: &str,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<PathBuf, String> {
    let module_root = sdk_root.join("plugins").join(module_name);
    let direct_project = module_root
        .join("project")
        .join(format!("{module_name}.xcodeproj"));
    if direct_project.is_dir() {
        return Ok(direct_project);
    }

    let projects = WalkDir::new(&module_root)
        .max_depth(5)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| {
            path.is_dir()
                && path
                    .extension()
                    .and_then(|value| value.to_str())
                    .is_some_and(|extension| extension == "xcodeproj")
        })
        .collect::<Vec<_>>();

    if let Some(project) = projects.iter().find(|path| {
        path.file_stem()
            .and_then(|value| value.to_str())
            .is_some_and(|stem| stem.eq_ignore_ascii_case(module_name))
    }) {
        return Ok(project.clone());
    }

    if let Some(project) = projects.first() {
        log_warn(
            log,
            job_id,
            "toolchain",
            &format!(
                "Using legacy Xcode project '{}' because '{}' was not generated",
                project.display(),
                direct_project.display()
            ),
        );
        return Ok(project.clone());
    }

    Err(format!(
        "Legacy Xcode project was not generated under {}",
        module_root.display()
    ))
}

#[cfg(target_os = "macos")]
#[derive(Deserialize)]
struct XcodeListOutput {
    project: XcodeProjectList,
}

#[cfg(target_os = "macos")]
#[derive(Deserialize)]
struct XcodeProjectList {
    schemes: Vec<String>,
}

#[cfg(target_os = "macos")]
fn resolve_legacy_xcode_scheme(
    project: &Path,
    module_name: &str,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<String, String> {
    let args = vec![
        "-list".to_string(),
        "-json".to_string(),
        "-project".to_string(),
        project.display().to_string(),
    ];
    log_command(
        log,
        job_id,
        &format!("Running: xcodebuild {}", args.join(" ")),
    );
    let output = Command::new("xcodebuild")
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("Failed to list Xcode schemes: {error}"))?;

    for line in String::from_utf8_lossy(&output.stderr).lines() {
        log_warn(log, job_id, "toolchain", line);
    }
    if !output.status.success() {
        return Err(format!(
            "Failed to list Xcode schemes with status {}",
            output.status
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    let list: XcodeListOutput = serde_json::from_str(&text)
        .map_err(|error| format!("Failed to parse Xcode scheme list: {error}"))?;
    let schemes = list.project.schemes;
    if schemes.is_empty() {
        return Err(format!(
            "No Xcode schemes were found in {}",
            project.display()
        ));
    }

    if let Some(scheme) = select_legacy_xcode_scheme(&schemes, module_name) {
        if !scheme.eq_ignore_ascii_case(module_name) {
            log_warn(
                log,
                job_id,
                "toolchain",
                &format!(
                    "Using legacy Xcode scheme '{scheme}' because '{module_name}' was not listed"
                ),
            );
        }
        return Ok(scheme);
    }

    Err(format!(
        "Xcode scheme '{module_name}' was not found. Available schemes: {}",
        schemes.join(", ")
    ))
}

#[cfg(target_os = "macos")]
fn select_legacy_xcode_scheme(schemes: &[String], module_name: &str) -> Option<String> {
    if let Some(scheme) = schemes
        .iter()
        .find(|scheme| scheme.eq_ignore_ascii_case(module_name))
    {
        return Some(scheme.clone());
    }

    if let Some(scheme) = schemes
        .iter()
        .find(|scheme| !scheme.ends_with(".framework"))
    {
        return Some(scheme.clone());
    }

    None
}

fn is_missing_legacy_generated_file_error(error: &str) -> bool {
    error.contains("Build input file cannot be found")
        && error.contains("generated/hxx")
        && error.contains("register.cpp")
}

#[cfg(target_os = "macos")]
fn prepare_python_shim() -> Result<PathBuf, String> {
    let shim_dir = local_data_root()?.join("legacy-python-shim");
    std::fs::create_dir_all(&shim_dir)
        .map_err(|error| format!("Failed to create {}: {error}", shim_dir.display()))?;
    let shim = shim_dir.join("python");
    if shim.exists() {
        std::fs::remove_file(&shim)
            .map_err(|error| format!("Failed to remove {}: {error}", shim.display()))?;
    }
    let python3 = find_usable_python3()
        .ok_or_else(|| "Python 3 was not found for legacy SDK build scripts".to_string())?;
    std::os::unix::fs::symlink(&python3, &shim)
        .map_err(|error| format!("Failed to create python shim: {error}"))?;
    Ok(shim_dir)
}

#[cfg(target_os = "macos")]
fn find_usable_python3() -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|dir| dir.join("python3"))
            .find(|candidate| candidate.is_file() && candidate != Path::new("/usr/bin/python3"))
    })
}

fn find_legacy_plugin_binary(
    sdk_root: &Path,
    configuration: &str,
    module_name: &str,
    binary_extension: &str,
) -> Result<PathBuf, String> {
    let expected_file_name = format!("{module_name}.{binary_extension}");
    let candidates = [
        sdk_root
            .join("build")
            .join(configuration)
            .join(&expected_file_name),
        sdk_root
            .join("plugins")
            .join(module_name)
            .join(&expected_file_name),
    ];
    if let Some(path) = candidates.iter().find(|path| path.is_file()) {
        return Ok(path.to_path_buf());
    }

    WalkDir::new(sdk_root.join("plugins").join(module_name))
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
                sdk_root.display()
            )
        })
}

#[cfg(target_os = "macos")]
fn run_command_with_path_prefix(
    program: &str,
    args: &[String],
    cwd: &Path,
    path_prefix: &Path,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<(), String> {
    log_command(
        log,
        job_id,
        &format!("Running: {program} {}", args.join(" ")),
    );
    let path = std::env::var("PATH").unwrap_or_default();
    let output = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env("PATH", format!("{}:{path}", path_prefix.display()))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|error| format!("Failed to run {program}: {error}"))?;

    for line in String::from_utf8_lossy(&output.stdout).lines() {
        log_command_output(log, job_id, program, "info", line);
    }
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        log_command_output(log, job_id, program, "warn", line);
    }

    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let output_text = format!("{stdout}\n{stderr}");
        Err(format!(
            "Command failed with status {}:\n{}",
            output.status,
            command_failure_summary(&output_text)
        ))
    }
}

fn tail_lines(text: &str, count: usize) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let start = lines.len().saturating_sub(count);
    lines[start..].join("\n")
}

fn command_failure_summary(text: &str) -> String {
    let diagnostic = diagnostic_context(text);
    if diagnostic.is_empty() {
        tail_lines(text, 80)
    } else {
        diagnostic
    }
}

fn diagnostic_context(text: &str) -> String {
    let lines = text.lines().collect::<Vec<_>>();
    let mut selected = Vec::<usize>::new();
    for (index, line) in lines.iter().enumerate() {
        if is_diagnostic_line(line) {
            let start = index.saturating_sub(2);
            let end = (index + 3).min(lines.len().saturating_sub(1));
            selected.extend(start..=end);
        }
    }

    selected.sort_unstable();
    selected.dedup();
    if selected.is_empty() {
        return String::new();
    }

    let mut result = Vec::new();
    let mut previous = None;
    for index in selected {
        if previous.is_some_and(|value| index > value + 1) {
            result.push("...".to_string());
        }
        result.push(lines[index].to_string());
        previous = Some(index);
    }
    result.join("\n")
}

fn is_diagnostic_line(line: &str) -> bool {
    line.contains("error:")
        || line.contains("fatal error:")
        || line.contains("** BUILD FAILED **")
        || line.contains("The following build commands failed:")
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    for entry in WalkDir::new(source)
        .into_iter()
        .filter_entry(|entry| should_copy_plugin_entry(source, entry.path()))
        .filter_map(Result::ok)
    {
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|error| error.to_string())?;
        let destination = target.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&destination)
                .map_err(|error| format!("Failed to create {}: {error}", destination.display()))?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
            }
            std::fs::copy(entry.path(), &destination).map_err(|error| {
                format!(
                    "Failed to copy {} to {}: {error}",
                    entry.path().display(),
                    destination.display()
                )
            })?;
        } else {
            continue;
        }
    }
    Ok(())
}

fn should_copy_plugin_entry(source: &Path, path: &Path) -> bool {
    if path == source {
        return true;
    }

    path.strip_prefix(source)
        .ok()
        .into_iter()
        .flat_map(|relative| relative.components())
        .filter_map(|component| component.as_os_str().to_str())
        .all(|name| !is_ignored_plugin_copy_name(name))
}

fn is_ignored_plugin_copy_name(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | ".hg"
            | ".svn"
            | ".cache"
            | ".pytest_cache"
            | ".idea"
            | ".vscode"
            | "__pycache__"
            | "build"
            | "dist"
            | "node_modules"
            | "target"
    )
}

fn remove_link_or_dir_cross_platform(path: &Path) -> Result<(), String> {
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

fn run_command(
    program: &str,
    args: &[String],
    cwd: &Path,
    log: LogCallback<'_>,
    job_id: &str,
) -> Result<(), String> {
    log_command(
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
        log_command_output(log, job_id, program, "info", line);
    }
    for line in String::from_utf8_lossy(&output.stderr).lines() {
        log_command_output(log, job_id, program, "warn", line);
    }

    if output.status.success() {
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let output_text = format!("{stdout}\n{stderr}");
        Err(format!(
            "Command failed with status {}:\n{}",
            output.status,
            command_failure_summary(&output_text)
        ))
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

fn log_sdk(log: LogCallback<'_>, job_id: &str, message: &str) {
    log_event(log, job_id, "info", "sdk", message);
}

fn log_package(log: LogCallback<'_>, job_id: &str, message: &str) {
    log_event(log, job_id, "info", "package", message);
}

fn log_command(log: LogCallback<'_>, job_id: &str, message: &str) {
    log_event(log, job_id, "info", "command", message);
}

fn log_command_output(
    log: LogCallback<'_>,
    job_id: &str,
    program: &str,
    level: &str,
    message: &str,
) {
    log_event(log, job_id, level, command_log_category(program), message);
}

fn log_warn(log: LogCallback<'_>, job_id: &str, category: &str, message: &str) {
    log_event(log, job_id, "warn", category, message);
}

fn log_event(log: LogCallback<'_>, job_id: &str, level: &str, category: &str, message: &str) {
    log(BuildLogEvent {
        job_id: job_id.to_string(),
        level: level.to_string(),
        category: category.to_string(),
        timestamp: build_log_timestamp(),
        message: message.to_string(),
    });
}

fn command_log_category(program: &str) -> &'static str {
    let name = Path::new(program)
        .file_name()
        .and_then(|file_name| file_name.to_str())
        .unwrap_or(program)
        .to_ascii_lowercase();

    if name.contains("cmake") {
        "cmake"
    } else if name.contains("xcodebuild") {
        "xcode"
    } else if name.contains("generate_solution") {
        "sdk"
    } else {
        "toolchain"
    }
}

pub fn build_log_timestamp() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default();
    millis.to_string()
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn cmake_target_name_removes_spaces() {
        assert_eq!(cmake_target_name("Boghma WaterMark"), "Boghma_WaterMark");
    }

    #[test]
    fn cmake_build_module_name_uses_requested_name_for_direct_module() {
        let plugin = TempTree::new("c4d-direct-plugin");
        std::fs::create_dir_all(plugin.path().join("project")).unwrap();
        std::fs::write(
            plugin.path().join("project").join("projectdefinition.txt"),
            "Type=DLL",
        )
        .unwrap();

        let log = |_event: BuildLogEvent| {};
        let resolved =
            resolve_cmake_build_module_name(plugin.path(), "BackHighlight", &log, "test").unwrap();

        assert_eq!(resolved, "BackHighlight");
    }

    #[test]
    fn cmake_build_module_name_uses_single_nested_projectdefinition_module() {
        let plugin = TempTree::new("c4d-nested-plugin");
        std::fs::create_dir_all(plugin.path().join("draw.back").join("project")).unwrap();
        std::fs::write(
            plugin
                .path()
                .join("draw.back")
                .join("project")
                .join("projectdefinition.txt"),
            "Type=DLL",
        )
        .unwrap();

        let log = |_event: BuildLogEvent| {};
        let resolved =
            resolve_cmake_build_module_name(plugin.path(), "BackHighlight", &log, "test").unwrap();

        assert_eq!(resolved, "draw.back");
    }

    #[test]
    fn command_failure_summary_prefers_compiler_errors() {
        let text = [
            "CompileC very long command",
            "/plugin/source/main.cpp:38:1: error: unknown type name 'Bool'",
            "Bool PluginStart()",
            "5 errors generated.",
            "** BUILD FAILED **",
            "The following build commands failed:",
            "CompileC main.o",
        ]
        .join("\n");

        let summary = command_failure_summary(&text);

        assert!(summary.contains("unknown type name 'Bool'"));
        assert!(summary.contains("** BUILD FAILED **"));
    }

    #[test]
    fn legacy_workspace_copy_skips_vcs_and_cache_dirs() {
        let source = TempTree::new("c4d-plugin-source");
        let target = TempTree::new("c4d-plugin-target");

        std::fs::create_dir_all(source.path().join("source")).unwrap();
        std::fs::write(source.path().join("source").join("main.cpp"), "plugin").unwrap();
        std::fs::create_dir_all(source.path().join(".git")).unwrap();
        std::fs::write(source.path().join(".git").join("config"), "git").unwrap();
        std::fs::create_dir_all(source.path().join("dist")).unwrap();
        std::fs::write(source.path().join("dist").join("old.zip"), "zip").unwrap();

        copy_dir_recursive(source.path(), target.path()).unwrap();

        assert!(target.path().join("source").join("main.cpp").is_file());
        assert!(!target.path().join(".git").exists());
        assert!(!target.path().join("dist").exists());
    }

    #[test]
    #[cfg(unix)]
    fn legacy_workspace_copy_skips_special_files() {
        use std::os::unix::net::UnixListener;

        let source = TempTree::new("c4d-plugin-source-socket");
        let target = TempTree::new("c4d-plugin-target-socket");

        std::fs::write(source.path().join("projectdefinition.txt"), "plugin").unwrap();
        let socket_path = source.path().join("fsmonitor--daemon.ipc");
        let listener = UnixListener::bind(&socket_path).unwrap();

        copy_dir_recursive(source.path(), target.path()).unwrap();

        drop(listener);
        assert!(target.path().join("projectdefinition.txt").is_file());
        assert!(!target.path().join("fsmonitor--daemon.ipc").exists());
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn legacy_xcode_scheme_matches_module_case_insensitively() {
        let schemes = vec![
            "boghma watermark".to_string(),
            "cinema.framework".to_string(),
        ];

        assert_eq!(
            select_legacy_xcode_scheme(&schemes, "Boghma WaterMark"),
            Some("boghma watermark".to_string())
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn legacy_xcode_scheme_falls_back_to_non_framework_scheme() {
        let schemes = vec![
            "cinema.framework".to_string(),
            "custom scheme".to_string(),
            "core.framework".to_string(),
        ];

        assert_eq!(
            select_legacy_xcode_scheme(&schemes, "Missing Module"),
            Some("custom scheme".to_string())
        );
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn legacy_xcode_project_falls_back_to_nested_generated_project() {
        let sdk = TempTree::new("c4d-legacy-sdk");
        let project = sdk
            .path()
            .join("plugins")
            .join("Draw.back")
            .join("draw.back")
            .join("project")
            .join("draw.back.xcodeproj");
        std::fs::create_dir_all(&project).unwrap();

        let log = |_event: BuildLogEvent| {};
        let resolved = resolve_legacy_xcode_project(sdk.path(), "Draw.back", &log, "test").unwrap();

        assert_eq!(resolved, project);
    }

    struct TempTree {
        path: PathBuf,
    }

    impl TempTree {
        fn new(name: &str) -> Self {
            let millis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before unix epoch")
                .as_millis();
            let path = PathBuf::from("/tmp").join(format!("{name}-{millis}"));
            std::fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}
