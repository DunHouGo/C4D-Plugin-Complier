//! C4D 插件编译器的 Tauri 命令。

use std::any::Any;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use tauri::{AppHandle, Manager};

use crate::compiler::env;
use crate::compiler::jobs::JobManager;
use crate::compiler::sdk;
use crate::types::{
    BuildArtifact, BuildJobId, BuildRequest, EnvironmentReport, SdkAutoConfigReport, SdkResolution,
    SdkRootConfig, SdkSetupReport, SdkSourceConfig, SdkSourceOverride, SdkVersionOption,
};

#[tauri::command]
#[specta::specta]
pub async fn detect_environment() -> Result<EnvironmentReport, String> {
    run_compiler_task(|| Ok(env::detect_environment())).await
}

#[tauri::command]
#[specta::specta]
pub async fn resolve_sdk_versions(request: BuildRequest) -> Result<Vec<SdkResolution>, String> {
    run_compiler_task(move || Ok(sdk::resolve_sdk_versions(&request))).await
}

#[tauri::command]
#[specta::specta]
pub async fn list_sdk_versions() -> Result<Vec<SdkVersionOption>, String> {
    run_compiler_task(|| Ok(sdk::available_sdk_versions())).await
}

#[tauri::command]
#[specta::specta]
pub async fn load_sdk_sources() -> Result<SdkSourceConfig, String> {
    run_compiler_task(sdk::load_sdk_source_config).await
}

#[tauri::command]
#[specta::specta]
pub async fn save_sdk_root_config(config: SdkRootConfig) -> Result<SdkSourceConfig, String> {
    run_compiler_task(move || sdk::save_sdk_root_config(config)).await
}

#[tauri::command]
#[specta::specta]
pub async fn auto_configure_sdk_sources() -> Result<SdkAutoConfigReport, String> {
    run_compiler_task(sdk::auto_configure_sdk_sources).await
}

#[tauri::command]
#[specta::specta]
pub async fn inspect_sdk_setup() -> Result<SdkSetupReport, String> {
    run_compiler_task(sdk::inspect_sdk_setup).await
}

#[tauri::command]
#[specta::specta]
pub async fn configure_required_sdks(
    config: SdkRootConfig,
    refresh: bool,
) -> Result<SdkSetupReport, String> {
    run_compiler_task(move || sdk::configure_required_sdks(config, refresh)).await
}

#[tauri::command]
#[specta::specta]
pub async fn save_sdk_source(
    version: String,
    source: SdkSourceOverride,
) -> Result<SdkVersionOption, String> {
    run_compiler_task(move || sdk::save_sdk_source(&version, source)).await
}

#[tauri::command]
#[specta::specta]
pub async fn remove_sdk_source(version: String) -> Result<Vec<SdkVersionOption>, String> {
    run_compiler_task(move || sdk::remove_sdk_source(&version)).await
}

#[tauri::command]
#[specta::specta]
pub async fn start_build(app: AppHandle, request: BuildRequest) -> Result<BuildJobId, String> {
    let manager = app.state::<JobManager>();
    Ok(manager.start_build(app.clone(), request))
}

#[tauri::command]
#[specta::specta]
pub async fn cancel_build(app: AppHandle, job_id: String) -> Result<bool, String> {
    let manager = app.state::<JobManager>();
    Ok(manager.cancel_build(&job_id))
}

#[tauri::command]
#[specta::specta]
pub async fn list_artifacts(app: AppHandle, job_id: String) -> Result<Vec<BuildArtifact>, String> {
    let manager = app.state::<JobManager>();
    Ok(manager.list_artifacts(&job_id))
}

#[tauri::command]
#[specta::specta]
pub async fn open_artifact_folder(path: String) -> Result<(), String> {
    let target = Path::new(&path);
    let folder = if target.is_file() {
        target
            .parent()
            .ok_or_else(|| format!("No parent folder for {path}"))?
    } else {
        target
    };

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(folder)
            .spawn()
            .map_err(|error| format!("Failed to open {}: {error}", folder.display()))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(folder)
            .spawn()
            .map_err(|error| format!("Failed to open {}: {error}", folder.display()))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(folder)
            .spawn()
            .map_err(|error| format!("Failed to open {}: {error}", folder.display()))?;
    }

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn save_build_log(path: String, contents: String) -> Result<(), String> {
    run_compiler_task(move || {
        let path = Path::new(&path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
        }

        std::fs::write(path, contents)
            .map_err(|error| format!("Failed to write {}: {error}", path.display()))
    })
    .await
}

async fn run_compiler_task<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        catch_unwind(AssertUnwindSafe(task))
            .map_err(|panic| format!("Compiler task panicked: {}", panic_message(&panic)))?
    })
    .await
    .map_err(|error| format!("Compiler task failed: {error}"))?
}

fn panic_message(panic: &Box<dyn Any + Send>) -> String {
    panic
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| panic.downcast_ref::<&str>().copied())
        .unwrap_or("unknown panic")
        .to_string()
}
