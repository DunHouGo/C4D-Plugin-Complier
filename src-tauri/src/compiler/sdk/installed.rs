//! 本机 Cinema 4D 安装与自带 sdk.zip 检测。

use std::path::{Path, PathBuf};

use crate::types::InstalledC4dVersion;

use super::versioning::{
    generated_download_url, installed_c4d_sdk_version, normalize_path_key, version_sort_key,
};

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

pub(super) fn installed_sdk_zip_path(version: &str) -> PathBuf {
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
