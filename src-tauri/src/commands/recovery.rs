//! 紧急数据恢复命令。
//!
//! 提供将 JSON 数据保存到磁盘的简单模式，
//! 用于崩溃恢复或会话持久化。

use serde_json::Value;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager};

use crate::types::{validate_filename, RecoveryError, MAX_RECOVERY_DATA_BYTES};

/// 获取恢复目录路径，必要时创建目录。
fn get_recovery_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {e}"))?;

    let recovery_dir = app_data_dir.join("recovery");

    // 确保恢复目录存在。
    std::fs::create_dir_all(&recovery_dir)
        .map_err(|e| format!("Failed to create recovery directory: {e}"))?;

    Ok(recovery_dir)
}

/// 将紧急数据保存为 JSON 文件，便于稍后恢复。
/// 校验文件名，并强制执行 10MB 大小限制。
#[tauri::command]
#[specta::specta]
pub async fn save_emergency_data(
    app: AppHandle,
    filename: String,
    data: Value,
) -> Result<(), RecoveryError> {
    log::info!("Saving emergency data to file: {filename}");

    // 使用安全规则校验文件名。
    validate_filename(&filename).map_err(|e| RecoveryError::ValidationError { message: e })?;

    // 只序列化一次格式化 JSON，同时用于大小校验和写入。
    let json_content = serde_json::to_string_pretty(&data).map_err(|e| {
        log::error!("Failed to serialize emergency data: {e}");
        RecoveryError::ParseError {
            message: e.to_string(),
        }
    })?;

    // 基于即将写入的实际内容检查 10MB 限制。
    if json_content.len() > MAX_RECOVERY_DATA_BYTES as usize {
        return Err(RecoveryError::DataTooLarge {
            max_bytes: MAX_RECOVERY_DATA_BYTES,
        });
    }

    let recovery_dir = get_recovery_dir(&app).map_err(|e| RecoveryError::IoError { message: e })?;
    let file_path = recovery_dir.join(format!("{filename}.json"));

    // 先写入临时文件，再重命名为目标文件，保证原子性。
    let temp_path = file_path.with_extension("tmp");

    std::fs::write(&temp_path, json_content).map_err(|e| {
        log::error!("Failed to write emergency data file: {e}");
        RecoveryError::IoError {
            message: e.to_string(),
        }
    })?;

    if let Err(rename_err) = std::fs::rename(&temp_path, &file_path) {
        log::error!("Failed to finalize emergency data file: {rename_err}");
        // 清理临时文件，避免磁盘上残留孤立文件。
        if let Err(remove_err) = std::fs::remove_file(&temp_path) {
            log::warn!("Failed to remove temp file after rename failure: {remove_err}");
        }
        return Err(RecoveryError::IoError {
            message: rename_err.to_string(),
        });
    }

    log::info!("Successfully saved emergency data to {file_path:?}");
    Ok(())
}

/// 从之前保存的 JSON 文件加载紧急数据。
/// 文件不存在时返回 FileNotFound。
#[tauri::command]
#[specta::specta]
pub async fn load_emergency_data(app: AppHandle, filename: String) -> Result<Value, RecoveryError> {
    log::info!("Loading emergency data from file: {filename}");

    // 使用安全规则校验文件名。
    validate_filename(&filename).map_err(|e| RecoveryError::ValidationError { message: e })?;

    let recovery_dir = get_recovery_dir(&app).map_err(|e| RecoveryError::IoError { message: e })?;
    let file_path = recovery_dir.join(format!("{filename}.json"));

    if !file_path.exists() {
        log::info!("Recovery file not found: {file_path:?}");
        return Err(RecoveryError::FileNotFound);
    }

    let contents = std::fs::read_to_string(&file_path).map_err(|e| {
        log::error!("Failed to read recovery file: {e}");
        RecoveryError::IoError {
            message: e.to_string(),
        }
    })?;

    let data: Value = serde_json::from_str(&contents).map_err(|e| {
        log::error!("Failed to parse recovery JSON: {e}");
        RecoveryError::ParseError {
            message: e.to_string(),
        }
    })?;

    log::info!("Successfully loaded emergency data");
    Ok(data)
}

/// 删除超过 7 天的恢复文件。
/// 返回已删除文件数量。
#[tauri::command]
#[specta::specta]
pub async fn cleanup_old_recovery_files(app: AppHandle) -> Result<u32, RecoveryError> {
    log::info!("Cleaning up old recovery files");

    let recovery_dir = get_recovery_dir(&app).map_err(|e| RecoveryError::IoError { message: e })?;
    let mut removed_count = 0;

    // 计算 7 天前的清理阈值。
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| RecoveryError::IoError {
            message: e.to_string(),
        })?
        .as_secs();
    let seven_days_ago = now - (7 * 24 * 60 * 60);

    // 读取目录并逐个检查文件。
    let entries = std::fs::read_dir(&recovery_dir).map_err(|e| {
        log::error!("Failed to read recovery directory: {e}");
        RecoveryError::IoError {
            message: e.to_string(),
        }
    })?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(e) => {
                log::warn!("Failed to read directory entry: {e}");
                continue;
            }
        };

        let path = entry.path();

        // 只处理 JSON 文件。
        if path.extension().is_none_or(|ext| ext != "json") {
            continue;
        }

        // 检查文件修改时间。
        let metadata = match std::fs::metadata(&path) {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to get file metadata: {e}");
                continue;
            }
        };

        let modified = match metadata.modified() {
            Ok(m) => m,
            Err(e) => {
                log::warn!("Failed to get file modification time: {e}");
                continue;
            }
        };

        let modified_secs = match modified.duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_secs(),
            Err(e) => {
                log::warn!("Failed to convert modification time: {e}");
                continue;
            }
        };

        // 超过 7 天则删除。
        if modified_secs < seven_days_ago {
            match std::fs::remove_file(&path) {
                Ok(_) => {
                    log::info!("Removed old recovery file: {path:?}");
                    removed_count += 1;
                }
                Err(e) => {
                    log::warn!("Failed to remove old recovery file: {e}");
                }
            }
        }
    }

    log::info!("Cleanup complete. Removed {removed_count} old recovery files");
    Ok(removed_count)
}
