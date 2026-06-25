//! SDK 来源配置文件读写和默认路径。

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::types::SdkSourceConfig;

use super::{
    is_cmake_sdk_root, is_legacy_sdk_root, load_sdk_source_config, SDK_CONFIG_FILE, SDK_ROOT_FOLDER,
};

pub(super) fn sdk_source_config_path() -> PathBuf {
    dirs::config_dir()
        .or_else(dirs::data_dir)
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."))
        .join("com.boghma.c4d-plugin-compiler")
        .join(SDK_CONFIG_FILE)
}

pub(super) fn legacy_workspace_sdk_source_config_paths() -> Vec<PathBuf> {
    vec![
        PathBuf::from("src-tauri")
            .join("configs")
            .join(SDK_CONFIG_FILE),
        PathBuf::from("configs").join(SDK_CONFIG_FILE),
    ]
}

pub(super) fn save_sdk_source_config(config: &SdkSourceConfig) -> Result<(), String> {
    let config_path = sdk_source_config_path();
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }
    let text = serde_json::to_string_pretty(config)
        .map_err(|error| format!("Failed to serialize SDK source config: {error}"))?;
    let temp_path = config_path.with_extension("json.tmp");
    std::fs::write(&temp_path, format!("{text}\n"))
        .map_err(|error| format!("Failed to write {}: {error}", temp_path.display()))?;
    std::fs::rename(&temp_path, &config_path).map_err(|error| {
        let _ = std::fs::remove_file(&temp_path);
        format!("Failed to write {}: {error}", config_path.display())
    })
}

pub(super) fn parse_sdk_source_config(text: &str) -> Result<SdkSourceConfig, String> {
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

pub(super) fn default_sdk_source_config() -> SdkSourceConfig {
    SdkSourceConfig {
        sdk_root: Some(default_sdk_root().display().to_string()),
    }
}

pub(super) fn configured_sdk_root() -> PathBuf {
    load_sdk_source_config()
        .ok()
        .and_then(|config| config.sdk_root)
        .map(PathBuf::from)
        .unwrap_or_else(default_sdk_root)
}

pub(super) fn configured_sdk_collection_root() -> PathBuf {
    let root = configured_sdk_root();
    if is_cmake_sdk_root(&root) || is_legacy_sdk_root(&root) {
        return root.parent().map(Path::to_path_buf).unwrap_or(root);
    }

    root
}

pub(super) fn default_sdk_root() -> PathBuf {
    dirs::document_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(SDK_ROOT_FOLDER)
}

pub(super) fn validate_no_spaces(path: &str) -> Result<(), String> {
    if path.chars().any(char::is_whitespace) {
        return Err(format!("SDK root must not contain spaces: {path}"));
    }
    Ok(())
}
