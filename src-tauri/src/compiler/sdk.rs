//! SDK discovery, download, cache, and extraction.

use std::collections::BTreeSet;
use std::fs::File;
use std::path::{Path, PathBuf};

use serde_json::Value;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::compiler::{parse_version_list, require_file};
use crate::types::{
    BuildRequest, InstalledC4dVersion, SdkAutoConfigReport, SdkResolution, SdkResolutionSource,
    SdkRootConfig, SdkSetupReport, SdkSourceConfig, SdkSourceOverride, SdkVersionOption,
};

const MAXON_BASE_URL: &str = "https://developers.maxon.net";
const SDK_ROOT_FOLDER: &str = "Maxon_SDK";
const SDK_CONFIG_FILE: &str = "sdk_sources.json";
pub const DEFAULT_MIN_SDK_VERSION: &str = "2024.4";

const KNOWN_SDK_VERSIONS: &[&str] = &["2024.4", "2025", "2026"];

pub fn available_sdk_versions() -> Vec<SdkVersionOption> {
    let config = load_sdk_source_config().unwrap_or_else(|_| default_sdk_source_config());
    let installed_versions = detect_installed_c4d_versions();
    let mut versions = BTreeSet::new();

    versions.extend(
        KNOWN_SDK_VERSIONS
            .iter()
            .map(|version| (*version).to_string()),
    );
    versions.extend(
        installed_versions
            .iter()
            .map(|item| item.sdk_version.clone()),
    );

    versions
        .into_iter()
        .filter(|version| version_sort_key(version) >= version_sort_key(DEFAULT_MIN_SDK_VERSION))
        .map(|version| sdk_version_option(&version, &config))
        .collect()
}

pub fn load_sdk_source_config() -> Result<SdkSourceConfig, String> {
    let config_path = sdk_source_config_path();
    if !config_path.is_file() {
        return Ok(default_sdk_source_config());
    }

    let text = std::fs::read_to_string(&config_path)
        .map_err(|error| format!("Failed to read {}: {error}", config_path.display()))?;
    parse_sdk_source_config(&text)
}

pub fn save_sdk_root_config(config: SdkRootConfig) -> Result<SdkSourceConfig, String> {
    let sdk_root = config
        .sdk_root
        .map(|item| item.trim().to_string())
        .filter(|item| !item.is_empty())
        .unwrap_or_else(|| default_sdk_root().display().to_string());
    validate_no_spaces(&sdk_root)?;

    let config = SdkSourceConfig {
        sdk_root: Some(sdk_root),
    };
    save_sdk_source_config(&config)?;
    Ok(config)
}

pub fn auto_configure_sdk_sources() -> Result<SdkAutoConfigReport, String> {
    let config = save_sdk_root_config(SdkRootConfig {
        sdk_root: Some(default_sdk_root().display().to_string()),
    })?;
    let installed_versions = detect_installed_c4d_versions();
    let versions = available_sdk_versions();

    Ok(SdkAutoConfigReport {
        sdk_root: config.sdk_root.clone(),
        installed_versions,
        versions,
    })
}

pub fn inspect_sdk_setup() -> Result<SdkSetupReport, String> {
    let config = load_sdk_source_config()?;
    Ok(sdk_setup_report(config.sdk_root, Vec::new()))
}

pub fn configure_required_sdks(
    config: SdkRootConfig,
    refresh: bool,
) -> Result<SdkSetupReport, String> {
    let config = save_sdk_root_config(config)?;
    let installed_versions = detect_installed_c4d_versions();
    let target_versions = required_sdk_versions(&installed_versions);
    let mut prepared_versions = Vec::new();

    for version in target_versions {
        prepared_versions.push(prepare_sdk(&version, refresh)?);
    }

    Ok(sdk_setup_report(config.sdk_root, prepared_versions))
}

pub fn save_sdk_source(
    version: &str,
    source: SdkSourceOverride,
) -> Result<SdkVersionOption, String> {
    let mut config = load_sdk_source_config()?;
    if let Some(root) = source.sdk_root {
        config = save_sdk_root_config(SdkRootConfig {
            sdk_root: Some(root),
        })?;
    } else if config.sdk_root.is_none() {
        config = save_sdk_root_config(SdkRootConfig { sdk_root: None })?;
    }

    Ok(sdk_version_option(version, &config))
}

pub fn remove_sdk_source(_version: &str) -> Result<Vec<SdkVersionOption>, String> {
    save_sdk_source_config(&default_sdk_source_config())?;
    Ok(available_sdk_versions())
}

pub fn resolve_sdk_versions(request: &BuildRequest) -> Vec<SdkResolution> {
    request
        .versions
        .iter()
        .flat_map(|version| parse_version_list(version))
        .map(|version| resolve_sdk_version(&version, request.refresh_sdk_cache))
        .collect()
}

pub fn prepare_sdk(version: &str, refresh: bool) -> Result<SdkResolution, String> {
    let resolution = resolve_sdk_version(version, refresh);
    if let Some(root) = &resolution.sdk_root {
        let path = PathBuf::from(root);
        if is_cmake_sdk_root(&path) || is_legacy_sdk_root(&path) {
            return Ok(resolution);
        }
    }

    match resolution.source {
        SdkResolutionSource::Config | SdkResolutionSource::InstalledZip => {
            let Some(archive_path) = resolution.archive_path.clone() else {
                return Err(format!("SDK {version} did not resolve to an archive path"));
            };
            match extract_sdk_archive(Path::new(&archive_path), version, refresh) {
                Ok(sdk_root) => Ok(SdkResolution {
                    sdk_root: Some(sdk_root.display().to_string()),
                    status: "ready".to_string(),
                    ..resolution
                }),
                Err(extract_error) => prepare_official_sdk(version, Some(extract_error)),
            }
        }
        SdkResolutionSource::OfficialDownload => prepare_official_sdk(version, None),
        SdkResolutionSource::Cache => Ok(resolution),
    }
}

fn prepare_official_sdk(
    version: &str,
    previous_error: Option<String>,
) -> Result<SdkResolution, String> {
    let Some(download_url) = find_official_cpp_sdk_url(version) else {
        return Err(previous_error
            .unwrap_or_else(|| format!("SDK {version} did not resolve to a download URL")));
    };

    let archive_path = download_sdk(&download_url, version)?;
    let sdk_root = extract_sdk_archive(&archive_path, version, true).map_err(|error| {
        previous_error
            .map(|previous| format!("{previous}; official SDK fallback failed: {error}"))
            .unwrap_or(error)
    })?;

    Ok(SdkResolution {
        version: version.to_string(),
        source: SdkResolutionSource::OfficialDownload,
        sdk_root: Some(sdk_root.display().to_string()),
        archive_path: Some(archive_path.display().to_string()),
        download_url: Some(download_url),
        status: "ready".to_string(),
    })
}

pub fn is_cmake_sdk_root(path: &Path) -> bool {
    path.join("CMakeLists.txt").is_file()
        && path.join("CMakePresets.json").is_file()
        && path.join("cmake").is_dir()
        && path.join("frameworks").is_dir()
        && path.join("plugins").is_dir()
}

pub fn is_legacy_sdk_root(path: &Path) -> bool {
    path.join("frameworks").is_dir()
        && path.join("plugins").is_dir()
        && path.join("tools").join("projecttool").is_dir()
        && (path.join("generate_solution_osx.command").is_file()
            || path.join("generate_solution_win.bat").is_file())
}

pub fn read_configure_presets(sdk_root: &Path) -> Result<Vec<String>, String> {
    let preset_path = sdk_root.join("CMakePresets.json");
    require_file(&preset_path)?;
    let text = std::fs::read_to_string(&preset_path)
        .map_err(|error| format!("Failed to read {}: {error}", preset_path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|error| format!("Failed to parse CMakePresets.json: {error}"))?;

    Ok(value
        .get("configurePresets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|item| item.get("name").and_then(Value::as_str))
        .map(str::to_string)
        .collect())
}

pub fn detect_installed_c4d_versions() -> Vec<InstalledC4dVersion> {
    let mut versions = detect_installed_c4d_versions_platform();
    versions.sort_by_key(|item| std::cmp::Reverse(version_sort_key(&item.sdk_version)));
    versions.dedup_by(|a, b| normalize_path_key(&a.path) == normalize_path_key(&b.path));
    versions
}

#[cfg(target_os = "windows")]
fn detect_installed_c4d_versions_platform() -> Vec<InstalledC4dVersion> {
    let mut versions = detect_installed_c4d_versions_registry();
    versions.extend(detect_installed_c4d_versions_windows_fallback());
    versions
}

#[cfg(target_os = "windows")]
fn detect_installed_c4d_versions_registry() -> Vec<InstalledC4dVersion> {
    use std::process::Command;

    let uninstall_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";
    let output = Command::new("reg")
        .args([
            "query",
            &format!(r"HKLM\{uninstall_path}"),
            "/s",
            "/f",
            "Maxon Cinema 4D",
            "/k",
        ])
        .output();

    let Ok(output) = output else {
        return Vec::new();
    };
    if !output.status.success() {
        return Vec::new();
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| line.ends_with("Maxon Cinema 4D") || line.contains("Maxon Cinema 4D "))
        .filter_map(registry_install_location)
        .filter_map(|path| installed_c4d_version_from_path(Path::new(&path)))
        .collect()
}

#[cfg(target_os = "windows")]
fn registry_install_location(key: &str) -> Option<String> {
    use std::process::Command;

    let output = Command::new("reg")
        .args(["query", key, "/v", "InstallLocation"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .find_map(|line| {
            let line = line.trim();
            if !line.starts_with("InstallLocation") {
                return None;
            }
            line.split_once("REG_SZ")
                .map(|(_, value)| value.trim().to_string())
                .filter(|value| !value.is_empty())
        })
}

#[cfg(target_os = "windows")]
fn detect_installed_c4d_versions_windows_fallback() -> Vec<InstalledC4dVersion> {
    let program_files = PathBuf::from(r"C:\Program Files\Maxon");
    if !program_files.is_dir() {
        return Vec::new();
    }

    std::fs::read_dir(&program_files)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .filter_map(|entry| installed_c4d_version_from_path(&entry.path()))
        .collect()
}

#[cfg(target_os = "macos")]
fn detect_installed_c4d_versions_platform() -> Vec<InstalledC4dVersion> {
    use std::collections::HashSet;
    use std::process::Command;

    let bundle_ids = ["net.maxon.cinema4d", "net.maxon.cinema4d.installer"];
    let mut versions = Vec::new();
    let mut seen_install_roots = HashSet::new();

    for bundle_id in bundle_ids {
        let Ok(output) = Command::new("mdfind")
            .arg(format!("kMDItemCFBundleIdentifier == '{bundle_id}'"))
            .output()
        else {
            continue;
        };
        let Ok(stdout) = String::from_utf8(output.stdout) else {
            continue;
        };

        for line in stdout.lines() {
            let path = Path::new(line.trim());
            if !path.exists() || !line.ends_with(".app") {
                continue;
            }
            let install_root = normalize_macos_install_root(path);
            let key = normalize_path_key(&install_root.display().to_string());
            if !seen_install_roots.insert(key) {
                continue;
            }
            if let Some(version) = installed_c4d_version_from_path(&install_root) {
                versions.push(version);
            }
        }
    }

    for version in detect_installed_c4d_versions_macos_fallback() {
        let key = normalize_path_key(&version.path);
        if seen_install_roots.insert(key) {
            versions.push(version);
        }
    }

    versions
}

#[cfg(target_os = "macos")]
fn detect_installed_c4d_versions_macos_fallback() -> Vec<InstalledC4dVersion> {
    let applications = PathBuf::from("/Applications");
    if !applications.is_dir() {
        return Vec::new();
    }

    std::fs::read_dir(&applications)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .filter_map(|entry| installed_c4d_version_from_path(&entry.path()))
        .collect()
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn detect_installed_c4d_versions_platform() -> Vec<InstalledC4dVersion> {
    Vec::new()
}

fn installed_c4d_version_from_path(path: &Path) -> Option<InstalledC4dVersion> {
    if !path.is_dir() {
        return None;
    }

    let name = path.file_name()?.to_string_lossy().to_string();
    if !name.to_lowercase().contains("cinema 4d") || name.to_lowercase().contains("installer") {
        return None;
    }

    let version = parse_c4d_version_id(&name);
    let sdk_version = installed_c4d_sdk_version(&version)?;
    Some(InstalledC4dVersion {
        version,
        path: path.display().to_string(),
        download_url: generated_download_url(&sdk_version),
        sdk_version,
    })
}

#[cfg(target_os = "macos")]
fn normalize_macos_install_root(path: &Path) -> PathBuf {
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("app"))
    {
        return path.parent().unwrap_or(path).to_path_buf();
    }

    path.to_path_buf()
}

fn parse_c4d_version_id(name: &str) -> String {
    let name = name.trim().trim_end_matches(".app").trim();
    for (index, character) in name.char_indices() {
        if matches!(character, 'R' | 'r' | 'S' | 's') && name[index..].len() >= 3 {
            let rest = &name[index + 1..index + 3];
            if rest.chars().all(|item| item.is_ascii_digit()) {
                return format!("{}{}", character.to_ascii_uppercase(), rest);
            }
        }
    }

    if let Some(position) = name.find("20") {
        if name[position..].len() >= 4 {
            let rest = &name[position..position + 4];
            if rest.chars().all(|item| item.is_ascii_digit()) {
                return rest.to_string();
            }
        }
    }

    name.to_string()
}

fn resolve_sdk_version(version: &str, refresh: bool) -> SdkResolution {
    let configured_root = configured_sdk_root();
    if let Some(sdk_root) = find_configured_sdk_root(&configured_root, version) {
        return SdkResolution {
            version: version.to_string(),
            source: SdkResolutionSource::Config,
            sdk_root: Some(sdk_root.display().to_string()),
            archive_path: None,
            download_url: generated_download_url_if_known(version),
            status: "configured root".to_string(),
        };
    }

    if let Some(archive_path) = find_configured_sdk_archive(&configured_root, version) {
        if sdk_archive_is_readable(&archive_path) {
            return SdkResolution {
                version: version.to_string(),
                source: SdkResolutionSource::Config,
                sdk_root: None,
                archive_path: Some(archive_path.display().to_string()),
                download_url: generated_download_url_if_known(version),
                status: "configured archive".to_string(),
            };
        }

        return SdkResolution {
            version: version.to_string(),
            source: SdkResolutionSource::OfficialDownload,
            sdk_root: None,
            archive_path: None,
            download_url: generated_download_url_if_known(version),
            status: format!("invalid configured archive: {}", archive_path.display()),
        };
    }

    let installed_zip = installed_sdk_zip_path(version);
    if installed_zip.is_file() {
        if sdk_archive_is_readable(&installed_zip) {
            return SdkResolution {
                version: version.to_string(),
                source: SdkResolutionSource::InstalledZip,
                sdk_root: None,
                archive_path: Some(installed_zip.display().to_string()),
                download_url: generated_download_url_if_known(version),
                status: "installed sdk.zip".to_string(),
            };
        }

        return SdkResolution {
            version: version.to_string(),
            source: SdkResolutionSource::OfficialDownload,
            sdk_root: None,
            archive_path: None,
            download_url: generated_download_url_if_known(version),
            status: format!("invalid installed sdk.zip: {}", installed_zip.display()),
        };
    }

    let sdk_root = sdk_cache_root(version);
    if sdk_root.is_dir() && !refresh && is_cmake_sdk_root(&sdk_root) {
        return SdkResolution {
            version: version.to_string(),
            source: SdkResolutionSource::Cache,
            sdk_root: Some(sdk_root.display().to_string()),
            archive_path: None,
            download_url: generated_download_url_if_known(version),
            status: "cached root".to_string(),
        };
    }

    if let Some(url) = generated_download_url_if_known(version) {
        return SdkResolution {
            version: version.to_string(),
            source: SdkResolutionSource::OfficialDownload,
            sdk_root: None,
            archive_path: None,
            download_url: Some(url),
            status: "auto download".to_string(),
        };
    }

    SdkResolution {
        version: version.to_string(),
        source: SdkResolutionSource::OfficialDownload,
        sdk_root: None,
        archive_path: None,
        download_url: generated_download_url_if_known(version),
        status: "no SDK source found".to_string(),
    }
}

fn find_official_cpp_sdk_url(version: &str) -> Option<String> {
    sdk_download_candidates(version).first().cloned()
}

fn find_configured_sdk_root(configured_root: &Path, version: &str) -> Option<PathBuf> {
    if !configured_root.is_dir() {
        return None;
    }

    if is_cmake_sdk_root(configured_root) || is_legacy_sdk_root(configured_root) {
        return path_matches_sdk_version(configured_root, version)
            .then(|| configured_root.to_path_buf());
    }

    WalkDir::new(configured_root)
        .max_depth(5)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| is_cmake_sdk_root(path) || is_legacy_sdk_root(path))
        .find(|path| path_matches_sdk_version(path, version))
}

fn find_configured_sdk_archive(configured_root: &Path, version: &str) -> Option<PathBuf> {
    if !configured_root.is_dir() {
        return None;
    }

    let expected_archive = sdk_archive_name(version);
    WalkDir::new(configured_root)
        .max_depth(5)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == expected_archive)
                || path_matches_sdk_version(path, version)
                    && path
                        .extension()
                        .and_then(|extension| extension.to_str())
                        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
        })
}

fn download_sdk(download_url: &str, version: &str) -> Result<PathBuf, String> {
    let archive_path = sdk_archive_path(version);
    if let Some(parent) = archive_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }

    if archive_path.is_file() {
        if sdk_archive_is_readable(&archive_path) {
            return Ok(archive_path);
        }

        std::fs::remove_file(&archive_path).map_err(|error| {
            format!(
                "Failed to remove invalid SDK archive {}: {error}",
                archive_path.display()
            )
        })?;
    }

    let mut response = reqwest::blocking::get(download_url)
        .map_err(|error| format!("Failed to download {download_url}: {error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to download {download_url}: HTTP {}",
            response.status()
        ));
    }

    let mut file = File::create(&archive_path)
        .map_err(|error| format!("Failed to create {}: {error}", archive_path.display()))?;
    response
        .copy_to(&mut file)
        .map_err(|error| format!("Failed to write {}: {error}", archive_path.display()))?;
    Ok(archive_path)
}

fn sdk_archive_is_readable(path: &Path) -> bool {
    File::open(path)
        .ok()
        .and_then(|file| ZipArchive::new(file).ok())
        .is_some()
}

fn extract_sdk_archive(
    archive_path: &Path,
    version: &str,
    refresh: bool,
) -> Result<PathBuf, String> {
    require_file(archive_path)?;
    let target_root = sdk_cache_root(version);
    if refresh && target_root.exists() {
        std::fs::remove_dir_all(&target_root)
            .map_err(|error| format!("Failed to remove {}: {error}", target_root.display()))?;
    }

    if target_root.exists() {
        if is_cmake_sdk_root(&target_root) || is_legacy_sdk_root(&target_root) {
            return Ok(target_root);
        }

        if let Some(nested) = find_nested_sdk_root(&target_root) {
            return Ok(nested);
        }

        std::fs::remove_dir_all(&target_root).map_err(|error| {
            format!(
                "Failed to remove incomplete SDK cache {}: {error}",
                target_root.display()
            )
        })?;
    }

    if !target_root.exists() {
        std::fs::create_dir_all(&target_root)
            .map_err(|error| format!("Failed to create {}: {error}", target_root.display()))?;
        let file = File::open(archive_path)
            .map_err(|error| format!("Failed to open {}: {error}", archive_path.display()))?;
        let mut archive =
            ZipArchive::new(file).map_err(|error| format!("Failed to read SDK zip: {error}"))?;
        archive
            .extract(&target_root)
            .map_err(|error| format!("Failed to extract SDK zip: {error}"))?;
    }

    if is_cmake_sdk_root(&target_root) || is_legacy_sdk_root(&target_root) {
        return Ok(target_root);
    }

    if let Some(nested) = find_nested_sdk_root(&target_root) {
        return Ok(nested);
    }

    Err(format!(
        "Extracted SDK root is incomplete or invalid: {}",
        target_root.display()
    ))
}

pub fn sdk_cache_root(version: &str) -> PathBuf {
    configured_sdk_collection_root()
        .join(version_folder(version))
        .join("sdk")
}

fn sdk_archive_path(version: &str) -> PathBuf {
    configured_sdk_collection_root()
        .join(version_folder(version))
        .join("downloads")
        .join(sdk_archive_name(version))
}

fn find_nested_sdk_root(root: &Path) -> Option<PathBuf> {
    WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_path_buf())
        .find(|path| is_cmake_sdk_root(path) || is_legacy_sdk_root(path))
}

fn sdk_source_config_path() -> PathBuf {
    PathBuf::from("configs").join(SDK_CONFIG_FILE)
}

fn save_sdk_source_config(config: &SdkSourceConfig) -> Result<(), String> {
    let config_path = sdk_source_config_path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(config)
        .map_err(|error| format!("Failed to serialize SDK source config: {error}"))?;
    std::fs::write(&config_path, format!("{text}\n"))
        .map_err(|error| format!("Failed to write {}: {error}", config_path.display()))
}

fn parse_sdk_source_config(text: &str) -> Result<SdkSourceConfig, String> {
    let value: Value = serde_json::from_str(text)
        .map_err(|error| format!("Failed to parse SDK config: {error}"))?;
    if value.get("sdk_root").is_some() {
        return serde_json::from_value::<SdkSourceConfig>(value)
            .map_err(|error| format!("Failed to parse SDK root config: {error}"));
    }
    if let Some(root) = legacy_sdk_root(&value) {
        return Ok(SdkSourceConfig {
            sdk_root: Some(root),
        });
    }
    Ok(default_sdk_source_config())
}

fn legacy_sdk_root(value: &Value) -> Option<String> {
    value.as_object().and_then(|object| {
        object.values().find_map(|entry| {
            entry.as_object().and_then(|item| {
                item.get("sdk_root")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .filter(|text| !text.trim().is_empty())
            })
        })
    })
}

fn default_sdk_source_config() -> SdkSourceConfig {
    SdkSourceConfig {
        sdk_root: Some(default_sdk_root().display().to_string()),
    }
}

fn configured_sdk_root() -> PathBuf {
    load_sdk_source_config()
        .ok()
        .and_then(|config| config.sdk_root)
        .map(PathBuf::from)
        .unwrap_or_else(default_sdk_root)
}

fn configured_sdk_collection_root() -> PathBuf {
    let root = configured_sdk_root();
    if is_cmake_sdk_root(&root) || is_legacy_sdk_root(&root) {
        return root.parent().map(Path::to_path_buf).unwrap_or(root);
    }

    root
}

fn default_sdk_root() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(SDK_ROOT_FOLDER)
}

fn validate_no_spaces(path: &str) -> Result<(), String> {
    if path.chars().any(char::is_whitespace) {
        return Err(format!("SDK root must not contain spaces: {path}"));
    }
    Ok(())
}

fn sdk_version_option(version: &str, config: &SdkSourceConfig) -> SdkVersionOption {
    let root = config
        .sdk_root
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(default_sdk_root);
    let download_url = generated_download_url_if_known(version);
    let configured = config.sdk_root.is_some();
    let resolved_sdk_root = find_configured_sdk_root(&root, version);
    let resolved_archive = find_configured_sdk_archive(&root, version);
    let installed_zip = installed_sdk_zip_path(version);
    let cached_root = sdk_cache_root(version);
    let (sdk_root, sdk_zip, status) = if let Some(sdk_root) = resolved_sdk_root {
        (
            Some(sdk_root.display().to_string()),
            None,
            "configured root".to_string(),
        )
    } else if let Some(archive_path) = resolved_archive {
        if !sdk_archive_is_readable(&archive_path) {
            (
                None,
                None,
                format!("invalid configured archive: {}", archive_path.display()),
            )
        } else {
            (
                None,
                Some(archive_path.display().to_string()),
                "configured archive".to_string(),
            )
        }
    } else if installed_zip.is_file() {
        if !sdk_archive_is_readable(&installed_zip) {
            (
                None,
                None,
                format!("invalid installed sdk.zip: {}", installed_zip.display()),
            )
        } else {
            (
                None,
                Some(installed_zip.display().to_string()),
                "installed sdk.zip".to_string(),
            )
        }
    } else if is_cmake_sdk_root(&cached_root) || is_legacy_sdk_root(&cached_root) {
        (
            Some(cached_root.display().to_string()),
            None,
            "cached root".to_string(),
        )
    } else {
        (
            None,
            None,
            download_url
                .as_ref()
                .map(|_| "auto download".to_string())
                .unwrap_or_else(|| "no SDK source found".to_string()),
        )
    };

    SdkVersionOption {
        version: version.to_string(),
        label: format!("Cinema 4D {version}"),
        configured,
        sdk_root,
        sdk_zip,
        download_url,
        status,
    }
}

fn sdk_setup_report(
    sdk_root: Option<String>,
    prepared_versions: Vec<SdkResolution>,
) -> SdkSetupReport {
    let installed_versions = detect_installed_c4d_versions();
    let versions = available_sdk_versions();
    let requirements = crate::compiler::env::setup_requirements(
        &installed_versions,
        &versions,
        sdk_root.as_deref(),
    );
    let missing_count = requirements
        .iter()
        .filter(|item| {
            matches!(
                item.status,
                crate::types::SetupRequirementStatus::Missing
                    | crate::types::SetupRequirementStatus::Manual
            )
        })
        .count();
    let summary = if missing_count == 0 {
        "Ready for Cinema 4D C++ SDK builds".to_string()
    } else {
        format!("{missing_count} setup items need attention")
    };

    SdkSetupReport {
        sdk_root,
        installed_versions,
        versions,
        prepared_versions,
        requirements,
        summary,
    }
}

fn required_sdk_versions(installed_versions: &[InstalledC4dVersion]) -> Vec<String> {
    let mut versions = BTreeSet::new();
    if installed_versions.is_empty() {
        versions.insert(DEFAULT_MIN_SDK_VERSION.to_string());
    } else {
        versions.extend(
            installed_versions
                .iter()
                .map(|item| item.sdk_version.clone())
                .filter(|version| {
                    version_sort_key(version) >= version_sort_key(DEFAULT_MIN_SDK_VERSION)
                }),
        );
    }

    versions.into_iter().collect()
}

fn sdk_download_candidates(version: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    if let Some(archive) = known_sdk_archive_name(version) {
        candidates.push(format!("{MAXON_BASE_URL}/downloads/{archive}"));
    }
    candidates
}

fn generated_download_url_if_known(version: &str) -> Option<String> {
    known_sdk_archive_name(version).map(|archive| format!("{MAXON_BASE_URL}/downloads/{archive}"))
}

fn generated_download_url(version: &str) -> String {
    generated_download_url_if_known(version).unwrap_or_else(|| {
        format!(
            "{MAXON_BASE_URL}/downloads/Cinema_4D_CPP_SDK_{}_0_0.zip",
            version.replace('.', "_")
        )
    })
}

fn sdk_archive_name(version: &str) -> String {
    known_sdk_archive_name(version)
        .map(str::to_string)
        .unwrap_or_else(|| format!("Cinema_4D_CPP_SDK_{}_0_0.zip", version.replace('.', "_")))
}

fn known_sdk_archive_name(version: &str) -> Option<&'static str> {
    match version {
        "2024" | "2024.4" => Some("Cinema_4D_CPP_SDK_2024_4_0.zip"),
        "2025" => Some("Cinema_4D_CPP_SDK_2025_0_1.zip"),
        "2026" => Some("Cinema_4D_CPP_SDK_2026_0_0.zip"),
        _ => None,
    }
}

fn installed_sdk_zip_path(version: &str) -> PathBuf {
    let install_version = if version == "2024.4" { "2024" } else { version };

    #[cfg(target_os = "macos")]
    {
        PathBuf::from(format!(
            "/Applications/Maxon Cinema 4D {install_version}/sdk.zip"
        ))
    }

    #[cfg(target_os = "windows")]
    {
        PathBuf::from(format!(
            r"C:\Program Files\Maxon Cinema 4D {install_version}\sdk.zip"
        ))
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let _ = install_version;
        PathBuf::new()
    }
}

fn installed_c4d_sdk_version(version: &str) -> Option<String> {
    let major = version.split('.').next()?;
    match major {
        "2024" => Some("2024.4".to_string()),
        "2025" => Some("2025".to_string()),
        "2026" => Some("2026".to_string()),
        _ => None,
    }
}

fn normalize_path_key(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}

fn path_matches_sdk_version(path: &Path, version: &str) -> bool {
    let path_text = normalize_path_key(&path.display().to_string());
    sdk_version_match_tokens(version)
        .iter()
        .any(|token| path_text.contains(token))
}

fn sdk_version_match_tokens(version: &str) -> Vec<String> {
    let major = version.split('.').next().unwrap_or(version);
    let mut tokens = vec![
        version.to_lowercase(),
        version.replace('.', "_").to_lowercase(),
        version.replace('.', "-").to_lowercase(),
    ];

    if version == major {
        tokens.push(format!("{major}_0_0").to_lowercase());
        tokens.push(format!("{major}-0-0").to_lowercase());
        tokens.push(format!("cinema_4d_cpp_sdk_{major}").to_lowercase());
        tokens.push(format!("cinema 4d {major}").to_lowercase());
    } else {
        tokens.push(format!("cinema_4d_cpp_sdk_{}", version.replace('.', "_")).to_lowercase());
        tokens.push(format!("cinema 4d {major}").to_lowercase());
    }

    tokens.sort();
    tokens.dedup();
    tokens
}

fn version_folder(version: &str) -> String {
    version.replace('.', "_")
}

fn version_sort_key(version: &str) -> u32 {
    let version = version.trim();
    if version.len() >= 2 && matches!(version.as_bytes()[0], b'R' | b'r') {
        return version[1..]
            .trim()
            .parse::<u32>()
            .map(|value| (2000 + value) * 100)
            .unwrap_or_default();
    }
    if version.len() >= 2 && matches!(version.as_bytes()[0], b'S' | b's') {
        return version[1..]
            .trim()
            .parse::<u32>()
            .map(|value| (2000 + value) * 100 + 1)
            .unwrap_or_default();
    }

    let mut parts = version.split('.');
    let major = parts
        .next()
        .and_then(|item| item.parse::<u32>().ok())
        .unwrap_or_default();
    let minor = parts
        .next()
        .and_then(|item| item.parse::<u32>().ok())
        .unwrap_or_default();
    major * 100 + minor
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::{
        find_configured_sdk_archive, find_configured_sdk_root, path_matches_sdk_version,
        sdk_version_match_tokens, sdk_version_option, SdkSourceConfig,
    };

    #[test]
    fn version_tokens_match_minor_sdk_names() {
        let tokens = sdk_version_match_tokens("2024.4");

        assert!(tokens.iter().any(|token| token == "2024.4"));
        assert!(tokens.iter().any(|token| token == "2024_4"));
        assert!(tokens
            .iter()
            .any(|token| token == "cinema_4d_cpp_sdk_2024_4"));
        assert!(path_matches_sdk_version(
            Path::new("/sdk/Cinema_4D_CPP_SDK_2024_4_0"),
            "2024.4"
        ));
    }

    #[test]
    fn configured_sdk_root_is_version_specific() {
        let root = TempTree::new("c4d-sdk-root");
        let sdk_2024 = root
            .path()
            .join("2024_4")
            .join("Cinema_4D_CPP_SDK_2024_4_0");
        let sdk_2026 = root
            .path()
            .join("2026_02")
            .join("Cinema_4D_CPP_SDK_2026_2_0");
        create_minimal_sdk_root(&sdk_2024);
        create_minimal_sdk_root(&sdk_2026);

        assert_eq!(
            find_configured_sdk_root(root.path(), "2024.4").as_deref(),
            Some(sdk_2024.as_path())
        );
        assert_eq!(
            find_configured_sdk_root(root.path(), "2026").as_deref(),
            Some(sdk_2026.as_path())
        );
        assert_eq!(find_configured_sdk_root(&sdk_2026, "2024.4"), None);
    }

    #[test]
    fn configured_sdk_archive_finds_version_archive() {
        let root = TempTree::new("c4d-sdk-archive");
        let archive = root
            .path()
            .join("downloads")
            .join("Cinema_4D_CPP_SDK_2024_4_0.zip");
        std::fs::create_dir_all(archive.parent().expect("archive parent")).unwrap();
        std::fs::write(&archive, []).unwrap();

        assert_eq!(
            find_configured_sdk_archive(root.path(), "2024.4").as_deref(),
            Some(archive.as_path())
        );
    }

    #[test]
    fn invalid_configured_archive_is_not_buildable() {
        let root = TempTree::new("c4d-sdk-invalid-archive");
        let archive = root
            .path()
            .join("downloads")
            .join("Cinema_4D_CPP_SDK_2025_0_1.zip");
        std::fs::create_dir_all(archive.parent().expect("archive parent")).unwrap();
        std::fs::write(&archive, "not a zip").unwrap();

        let option = sdk_version_option(
            "2025",
            &SdkSourceConfig {
                sdk_root: Some(root.path().display().to_string()),
            },
        );

        assert!(option.status.starts_with("invalid configured archive:"));
        assert_eq!(option.sdk_zip, None);
    }

    #[test]
    fn available_versions_keep_known_sdk_range() {
        let versions = super::available_sdk_versions()
            .into_iter()
            .map(|version| version.version)
            .collect::<Vec<_>>();

        assert!(versions.contains(&"2024.4".to_string()));
        assert!(versions.contains(&"2025".to_string()));
        assert!(versions.contains(&"2026".to_string()));
    }

    fn create_minimal_sdk_root(path: &Path) {
        std::fs::create_dir_all(path.join("cmake")).unwrap();
        std::fs::create_dir_all(path.join("frameworks")).unwrap();
        std::fs::create_dir_all(path.join("plugins")).unwrap();
        std::fs::write(path.join("CMakeLists.txt"), "").unwrap();
        std::fs::write(path.join("CMakePresets.json"), "{}").unwrap();
    }

    struct TempTree {
        path: std::path::PathBuf,
    }

    impl TempTree {
        fn new(name: &str) -> Self {
            let millis = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system clock before unix epoch")
                .as_millis();
            let path = std::env::temp_dir().join(format!("{name}-{millis}"));
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
