//! 崩溃诊断和持久日志命令。

use std::backtrace::Backtrace;
use std::fs::OpenOptions;
use std::io::Write;
use std::panic::PanicHookInfo;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, Manager};

/// 崩溃日志文件名。
const CRASH_LOG_FILE: &str = "crash.log";

/// 获取应用日志目录，必要时创建目录。
pub fn app_log_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|error| format!("Failed to get app log directory: {error}"))?;
    std::fs::create_dir_all(&log_dir).map_err(|error| {
        format!(
            "Failed to create log directory {}: {error}",
            log_dir.display()
        )
    })?;
    Ok(log_dir)
}

/// 获取日志目录路径，方便用户定位崩溃日志。
#[tauri::command]
#[specta::specta]
pub async fn get_log_dir(app: AppHandle) -> Result<String, String> {
    app_log_dir(&app).map(|path| path.display().to_string())
}

/// 追加一条前端或后端崩溃日志。
#[tauri::command]
#[specta::specta]
pub async fn append_crash_log(
    app: AppHandle,
    source: String,
    message: String,
    stack: Option<String>,
    context: Option<serde_json::Value>,
) -> Result<String, String> {
    append_crash_log_entry(&app, &source, &message, stack.as_deref(), context.as_ref())
}

/// 注册 Rust panic hook，尽量在进程退出前落盘。
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let message = panic_message(panic_info);
        let backtrace = Backtrace::force_capture().to_string();
        if let Err(error) = append_process_crash_log("rust-panic", &message, Some(&backtrace)) {
            eprintln!("Failed to write crash log: {error}");
        }
        default_hook(panic_info);
    }));
}

fn append_process_crash_log(
    source: &str,
    message: &str,
    stack: Option<&str>,
) -> Result<String, String> {
    let log_dir = process_log_dir()?;
    append_crash_log_to_dir(&log_dir, source, message, stack, None)
}

fn append_crash_log_entry(
    app: &AppHandle,
    source: &str,
    message: &str,
    stack: Option<&str>,
    context: Option<&serde_json::Value>,
) -> Result<String, String> {
    let log_dir = app_log_dir(app)?;
    append_crash_log_to_dir(&log_dir, source, message, stack, context)
}

fn append_crash_log_to_dir(
    log_dir: &PathBuf,
    source: &str,
    message: &str,
    stack: Option<&str>,
    context: Option<&serde_json::Value>,
) -> Result<String, String> {
    std::fs::create_dir_all(log_dir).map_err(|error| {
        format!(
            "Failed to create crash log directory {}: {error}",
            log_dir.display()
        )
    })?;
    let log_path = log_dir.join(CRASH_LOG_FILE);
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .map_err(|error| format!("Failed to open crash log {}: {error}", log_path.display()))?;

    let mut entry = format!("===== {} [{}] =====\n{message}\n", timestamp(), source);
    if let Some(context) = context {
        entry.push_str(&format!("context: {context}\n"));
    }
    if let Some(stack) = stack.filter(|text| !text.trim().is_empty()) {
        entry.push_str(&format!("stack:\n{stack}\n"));
    }
    entry.push('\n');
    file.write_all(entry.as_bytes())
        .map_err(|error| format!("Failed to write crash log entry: {error}"))?;

    Ok(log_path.display().to_string())
}

fn process_log_dir() -> Result<PathBuf, String> {
    dirs::cache_dir()
        .or_else(dirs::data_dir)
        .or_else(dirs::home_dir)
        .map(|path| path.join("C4D Plugin Compiler").join("logs"))
        .ok_or_else(|| "Failed to resolve fallback crash log directory".to_string())
}

fn timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn panic_message(panic_info: &PanicHookInfo<'_>) -> String {
    let location = panic_info
        .location()
        .map(|location| format!("{}:{}", location.file(), location.line()))
        .unwrap_or_else(|| "unknown location".to_string());
    let payload = panic_info
        .payload()
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic_info.payload().downcast_ref::<&str>().copied())
        .unwrap_or("unknown panic");

    format!("{payload} at {location}")
}
