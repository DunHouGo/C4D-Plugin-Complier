//! Tauri commands for the C4D plugin compiler.

use std::path::Path;

use tauri::{AppHandle, Manager};

use crate::compiler::env;
use crate::compiler::jobs::JobManager;
use crate::compiler::sdk;
use crate::types::{BuildArtifact, BuildJobId, BuildRequest, EnvironmentReport, SdkResolution};

#[tauri::command]
#[specta::specta]
pub async fn detect_environment() -> Result<EnvironmentReport, String> {
    Ok(env::detect_environment())
}

#[tauri::command]
#[specta::specta]
pub async fn resolve_sdk_versions(request: BuildRequest) -> Result<Vec<SdkResolution>, String> {
    Ok(sdk::resolve_sdk_versions(&request))
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
