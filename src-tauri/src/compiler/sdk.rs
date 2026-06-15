//! SDK discovery, download, cache, and extraction.

use std::fs::File;
use std::path::{Path, PathBuf};

use serde_json::Value;
use walkdir::WalkDir;
use zip::ZipArchive;

use crate::compiler::{local_data_root, parse_version_list, require_file};
use crate::types::{BuildRequest, SdkResolution, SdkResolutionSource, SdkSourceConfig};

const MAXON_DOWNLOADS_JSON: &str = "https://developers.maxon.net/src/downloads.json?1015";
const MAXON_BASE_URL: &str = "https://developers.maxon.net";

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
        if is_cmake_sdk_root(&path) {
            return Ok(resolution);
        }
    }

    match resolution.source {
        SdkResolutionSource::Config | SdkResolutionSource::InstalledZip => {
            let Some(archive_path) = resolution.archive_path.clone() else {
                return Err(format!("SDK {version} did not resolve to an archive path"));
            };
            let sdk_root = extract_sdk_archive(Path::new(&archive_path), version, refresh)?;
            Ok(SdkResolution {
                sdk_root: Some(sdk_root.display().to_string()),
                status: "ready".to_string(),
                ..resolution
            })
        }
        SdkResolutionSource::OfficialDownload => {
            let Some(download_url) = resolution.download_url.clone() else {
                return Err(format!("SDK {version} did not resolve to a download URL"));
            };
            let archive_path = download_sdk(&download_url, version)?;
            let sdk_root = extract_sdk_archive(&archive_path, version, refresh)?;
            Ok(SdkResolution {
                archive_path: Some(archive_path.display().to_string()),
                sdk_root: Some(sdk_root.display().to_string()),
                status: "ready".to_string(),
                ..resolution
            })
        }
        SdkResolutionSource::Cache => Ok(resolution),
    }
}

pub fn is_cmake_sdk_root(path: &Path) -> bool {
    path.join("CMakeLists.txt").is_file()
        && path.join("CMakePresets.json").is_file()
        && path.join("cmake").is_dir()
        && path.join("frameworks").is_dir()
        && path.join("plugins").is_dir()
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

fn resolve_sdk_version(version: &str, refresh: bool) -> SdkResolution {
    let cache_root = sdk_cache_root(version);
    if cache_root.is_dir() && !refresh && is_cmake_sdk_root(&cache_root) {
        return SdkResolution {
            version: version.to_string(),
            source: SdkResolutionSource::Cache,
            sdk_root: Some(cache_root.display().to_string()),
            archive_path: None,
            download_url: None,
            status: "cached".to_string(),
        };
    }

    if let Some(config) = load_sdk_source_config().ok().and_then(|config| {
        config
            .get(version)
            .cloned()
            .or_else(|| config.get(&format!("Cinema 4D {version}")).cloned())
    }) {
        if let Some(root) = config.sdk_root.as_ref().map(PathBuf::from) {
            if is_cmake_sdk_root(&root) {
                return SdkResolution {
                    version: version.to_string(),
                    source: SdkResolutionSource::Config,
                    sdk_root: Some(root.display().to_string()),
                    archive_path: None,
                    download_url: config.download_url,
                    status: "configured root".to_string(),
                };
            }
        }
        if let Some(zip) = config.sdk_zip.as_ref().map(PathBuf::from) {
            if zip.is_file() {
                return SdkResolution {
                    version: version.to_string(),
                    source: SdkResolutionSource::Config,
                    sdk_root: None,
                    archive_path: Some(zip.display().to_string()),
                    download_url: config.download_url,
                    status: "configured archive".to_string(),
                };
            }
        }
        if let Some(download_url) = config.download_url {
            return SdkResolution {
                version: version.to_string(),
                source: SdkResolutionSource::OfficialDownload,
                sdk_root: None,
                archive_path: None,
                download_url: Some(download_url),
                status: "configured download".to_string(),
            };
        }
    }

    let installed_zip = PathBuf::from(format!(
        r"C:\Program Files\Maxon Cinema 4D {version}\sdk.zip"
    ));
    if installed_zip.is_file() {
        return SdkResolution {
            version: version.to_string(),
            source: SdkResolutionSource::InstalledZip,
            sdk_root: None,
            archive_path: Some(installed_zip.display().to_string()),
            download_url: None,
            status: "installed sdk.zip".to_string(),
        };
    }

    match find_official_cpp_sdk_url(version) {
        Ok(url) => SdkResolution {
            version: version.to_string(),
            source: SdkResolutionSource::OfficialDownload,
            sdk_root: None,
            archive_path: None,
            download_url: Some(url),
            status: "official download".to_string(),
        },
        Err(error) => SdkResolution {
            version: version.to_string(),
            source: SdkResolutionSource::OfficialDownload,
            sdk_root: None,
            archive_path: None,
            download_url: None,
            status: error,
        },
    }
}

fn load_sdk_source_config() -> Result<SdkSourceConfig, String> {
    let config_path = PathBuf::from("configs").join("sdk_sources.json");
    if !config_path.is_file() {
        return Ok(SdkSourceConfig::new());
    }

    let text = std::fs::read_to_string(&config_path)
        .map_err(|error| format!("Failed to read {}: {error}", config_path.display()))?;
    serde_json::from_str::<SdkSourceConfig>(&text)
        .map_err(|error| format!("Failed to parse {}: {error}", config_path.display()))
}

fn find_official_cpp_sdk_url(version: &str) -> Result<String, String> {
    let data: Value = reqwest::blocking::get(MAXON_DOWNLOADS_JSON)
        .map_err(|error| format!("Failed to fetch Maxon downloads metadata: {error}"))?
        .json()
        .map_err(|error| format!("Failed to parse Maxon downloads metadata: {error}"))?;

    let releases = data
        .get("c4d")
        .and_then(|item| item.get("releases"))
        .and_then(Value::as_array)
        .ok_or_else(|| "Maxon downloads metadata did not contain c4d releases".to_string())?;

    let release = releases
        .iter()
        .find(|release| {
            release
                .get("label")
                .and_then(Value::as_str)
                .is_some_and(|label| label.contains(version))
        })
        .ok_or_else(|| format!("No official Cinema 4D {version} SDK entry was found"))?;

    let rows = release
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| format!("Cinema 4D {version} SDK entry did not contain rows"))?;

    for row in rows {
        for item in row.as_array().into_iter().flatten() {
            let is_cpp_sdk = item
                .get("label")
                .and_then(Value::as_str)
                .is_some_and(|label| label == "C++ SDK");
            if is_cpp_sdk {
                if let Some(url) = item.get("url").and_then(Value::as_str) {
                    if url.starts_with("http") {
                        return Ok(url.to_string());
                    }
                    return Ok(format!("{MAXON_BASE_URL}{url}"));
                }
            }
        }
    }

    Err(format!(
        "No official Cinema 4D {version} C++ SDK download URL was found"
    ))
}

fn download_sdk(download_url: &str, version: &str) -> Result<PathBuf, String> {
    let archive_dir = local_data_root()?.join("downloads");
    std::fs::create_dir_all(&archive_dir)
        .map_err(|error| format!("Failed to create {}: {error}", archive_dir.display()))?;
    let archive_path = archive_dir.join(format!("Cinema4D_{version}_CPP_SDK.zip"));

    if archive_path.is_file() {
        return Ok(archive_path);
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

    if is_cmake_sdk_root(&target_root) {
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
    local_data_root()
        .unwrap_or_else(|_| {
            PathBuf::from(".cache")
                .join("Boghma")
                .join("C4DPluginCompiler")
        })
        .join("sdks")
        .join(format!("Cinema4D_{version}"))
}

fn find_nested_sdk_root(root: &Path) -> Option<PathBuf> {
    WalkDir::new(root)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_path_buf())
        .find(|path| is_cmake_sdk_root(path))
}
