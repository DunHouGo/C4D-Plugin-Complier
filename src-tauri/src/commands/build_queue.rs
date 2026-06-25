//! 构建队列预设持久化命令。
//!
//! 使用应用数据目录中的 JSON 文件保存队列预设，避免仅依赖前端存储。

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::types::{BuildQueuePresetStore, RecoveryError, MAX_RECOVERY_DATA_BYTES};

const BUILD_QUEUE_PRESETS_FILE: &str = "build_queue_presets.json";

fn get_build_queue_presets_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Failed to get app data directory: {error}"))?;
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|error| format!("Failed to create app data directory: {error}"))?;
    Ok(app_data_dir.join(BUILD_QUEUE_PRESETS_FILE))
}

/// 从磁盘加载构建队列预设。
/// 文件不存在时返回空集合。
#[tauri::command]
#[specta::specta]
pub async fn load_build_queue_presets(
    app: AppHandle,
) -> Result<BuildQueuePresetStore, RecoveryError> {
    let path =
        get_build_queue_presets_path(&app).map_err(|message| RecoveryError::IoError { message })?;
    if !path.exists() {
        return Ok(BuildQueuePresetStore::default());
    }

    let contents = std::fs::read_to_string(&path).map_err(|error| RecoveryError::IoError {
        message: error.to_string(),
    })?;
    let store: BuildQueuePresetStore =
        serde_json::from_str(&contents).map_err(|error| RecoveryError::ParseError {
            message: error.to_string(),
        })?;
    Ok(store)
}

/// 将构建队列预设保存到磁盘。
/// 使用临时文件加重命名的原子写入方式。
#[tauri::command]
#[specta::specta]
pub async fn save_build_queue_presets(
    app: AppHandle,
    store: BuildQueuePresetStore,
) -> Result<(), RecoveryError> {
    let path =
        get_build_queue_presets_path(&app).map_err(|message| RecoveryError::IoError { message })?;
    let json_content =
        serde_json::to_string_pretty(&store).map_err(|error| RecoveryError::ParseError {
            message: error.to_string(),
        })?;
    if json_content.len() > MAX_RECOVERY_DATA_BYTES as usize {
        return Err(RecoveryError::DataTooLarge {
            max_bytes: MAX_RECOVERY_DATA_BYTES,
        });
    }

    let temp_path = path.with_extension("tmp");
    std::fs::write(&temp_path, json_content).map_err(|error| RecoveryError::IoError {
        message: error.to_string(),
    })?;

    if let Err(rename_err) = std::fs::rename(&temp_path, &path) {
        if let Err(remove_err) = std::fs::remove_file(&temp_path) {
            log::warn!("Failed to remove temp file after rename failure: {remove_err}");
        }
        return Err(RecoveryError::IoError {
            message: rename_err.to_string(),
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        BuildConfiguration, BuildQueuePreset, BuildRequest, PackageMode, SdkSourceMode,
    };

    #[test]
    fn build_queue_preset_store_serializes() {
        let store = BuildQueuePresetStore {
            presets: vec![BuildQueuePreset {
                id: "preset-1".to_string(),
                name: "Queue".to_string(),
                requests: vec![BuildRequest {
                    plugin_root: "/tmp/plugin".to_string(),
                    module_name: "Plugin".to_string(),
                    package_name: "Plugin".to_string(),
                    versions: vec!["2026".to_string()],
                    configuration: BuildConfiguration::Release,
                    sdk_source: SdkSourceMode::ConfiguredThenInstalledThenOfficial,
                    package_mode: PackageMode::Both,
                    zip_enabled: true,
                    clean_output: true,
                    refresh_sdk_cache: false,
                    output_dir: None,
                }],
                created_at: "2026-06-25T00:00:00.000Z".to_string(),
            }],
        };

        let json = serde_json::to_string(&store).unwrap();

        assert!(json.contains("\"created_at\""));
        assert!(json.contains("\"preset-1\""));
    }
}
