//! 偏好设置管理命令。
//!
//! 负责从磁盘加载和保存用户偏好设置。

use std::path::PathBuf;
use tauri::{AppHandle, Manager};

use crate::types::{validate_string_input, validate_theme, AppPreferences};

/// 获取偏好设置文件路径。
fn get_preferences_path(app: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {e}"))?;

    // 确保目录存在。
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {e}"))?;

    Ok(app_data_dir.join("preferences.json"))
}

/// 示例问候命令。
#[tauri::command]
#[specta::specta]
pub fn greet(name: &str) -> Result<String, String> {
    // 输入校验。
    validate_string_input(name, 100, "Name").map_err(|e| {
        log::warn!("Invalid greet input: {e}");
        e
    })?;

    log::info!("Greeting user: {name}");
    Ok(format!("Hello, {name}! You've been greeted from Rust!"))
}

/// 从磁盘加载用户偏好设置。
/// 文件不存在时返回默认偏好设置。
#[tauri::command]
#[specta::specta]
pub async fn load_preferences(app: AppHandle) -> Result<AppPreferences, String> {
    log::debug!("Loading preferences from disk");
    let prefs_path = get_preferences_path(&app)?;

    if !prefs_path.exists() {
        log::info!("Preferences file not found, using defaults");
        return Ok(AppPreferences::default());
    }

    let contents = std::fs::read_to_string(&prefs_path).map_err(|e| {
        log::error!("Failed to read preferences file: {e}");
        format!("Failed to read preferences file: {e}")
    })?;

    let preferences: AppPreferences = serde_json::from_str(&contents).map_err(|e| {
        log::error!("Failed to parse preferences JSON: {e}");
        format!("Failed to parse preferences: {e}")
    })?;

    log::info!("Successfully loaded preferences");
    Ok(preferences)
}

/// 将用户偏好设置保存到磁盘。
/// 使用临时文件加重命名的原子写入方式，避免文件损坏。
#[tauri::command]
#[specta::specta]
pub async fn save_preferences(app: AppHandle, preferences: AppPreferences) -> Result<(), String> {
    // 校验主题取值。
    validate_theme(&preferences.theme)?;

    log::debug!("Saving preferences to disk: {preferences:?}");
    let prefs_path = get_preferences_path(&app)?;

    let json_content = serde_json::to_string_pretty(&preferences).map_err(|e| {
        log::error!("Failed to serialize preferences: {e}");
        format!("Failed to serialize preferences: {e}")
    })?;

    // 先写入临时文件，再重命名为目标文件，保证原子性。
    let temp_path = prefs_path.with_extension("tmp");

    std::fs::write(&temp_path, json_content).map_err(|e| {
        log::error!("Failed to write preferences file: {e}");
        format!("Failed to write preferences file: {e}")
    })?;

    if let Err(rename_err) = std::fs::rename(&temp_path, &prefs_path) {
        log::error!("Failed to finalize preferences file: {rename_err}");
        // 清理临时文件，避免磁盘上残留孤立文件。
        if let Err(remove_err) = std::fs::remove_file(&temp_path) {
            log::warn!("Failed to remove temp file after rename failure: {remove_err}");
        }
        return Err(format!("Failed to finalize preferences file: {rename_err}"));
    }

    log::info!("Successfully saved preferences to {prefs_path:?}");
    Ok(())
}
