//! Packaging helpers for compiled Cinema 4D plugin binaries.

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use zip::write::FileOptions;

use crate::compiler::require_dir;
use crate::types::{BuildArtifact, BuildRequest, PackageMode};

pub fn package_outputs(
    request: &BuildRequest,
    built_binaries: &[(String, String, PathBuf)],
) -> Result<Vec<BuildArtifact>, String> {
    let plugin_root = PathBuf::from(&request.plugin_root);
    require_dir(&plugin_root)?;
    let output_root = request
        .output_dir
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| plugin_root.join("dist"));

    if request.clean_output && output_root.exists() {
        remove_path(&output_root)?;
    }
    std::fs::create_dir_all(&output_root)
        .map_err(|error| format!("Failed to create {}: {error}", output_root.display()))?;

    let mut artifacts = Vec::new();
    if matches!(
        request.package_mode,
        PackageMode::Merged | PackageMode::Both
    ) {
        artifacts.extend(create_merged_package(
            request,
            built_binaries,
            &output_root,
        )?);
    }
    if matches!(
        request.package_mode,
        PackageMode::PerVersion | PackageMode::Both
    ) {
        artifacts.extend(create_per_version_packages(
            request,
            built_binaries,
            &output_root,
        )?);
    }

    Ok(artifacts)
}

fn create_merged_package(
    request: &BuildRequest,
    built_binaries: &[(String, String, PathBuf)],
    output_root: &Path,
) -> Result<Vec<BuildArtifact>, String> {
    let package_dir = output_root.join(&request.package_name);
    if request.clean_output && package_dir.exists() {
        remove_path(&package_dir)?;
    }
    std::fs::create_dir_all(&package_dir)
        .map_err(|error| format!("Failed to create {}: {error}", package_dir.display()))?;

    let plugin_root = Path::new(&request.plugin_root);
    for (version, configuration, binary) in built_binaries {
        copy_resources(plugin_root, binary, &package_dir)?;
        let suffix = binary
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!(".{value}"))
            .unwrap_or_default();
        let target_name =
            package_binary_name(&request.package_name, version, configuration, &suffix);
        std::fs::copy(binary, package_dir.join(target_name))
            .map_err(|error| format!("Failed to copy {}: {error}", binary.display()))?;
    }

    let mut artifacts = vec![BuildArtifact {
        version: None,
        configuration: None,
        kind: "merged-package".to_string(),
        path: package_dir.display().to_string(),
    }];

    if request.zip_enabled {
        let zip_path = create_zip_archive(&package_dir)?;
        artifacts.push(BuildArtifact {
            version: None,
            configuration: None,
            kind: "merged-zip".to_string(),
            path: zip_path.display().to_string(),
        });
    }

    Ok(artifacts)
}

fn create_per_version_packages(
    request: &BuildRequest,
    built_binaries: &[(String, String, PathBuf)],
    output_root: &Path,
) -> Result<Vec<BuildArtifact>, String> {
    let mut artifacts = Vec::new();

    for (version, configuration, binary) in built_binaries {
        let package_dir = output_root.join(package_folder_name(
            &request.package_name,
            version,
            configuration,
        ));
        if request.clean_output && package_dir.exists() {
            remove_path(&package_dir)?;
        }
        std::fs::create_dir_all(&package_dir)
            .map_err(|error| format!("Failed to create {}: {error}", package_dir.display()))?;
        copy_resources(Path::new(&request.plugin_root), binary, &package_dir)?;

        let suffix = binary
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!(".{value}"))
            .unwrap_or_default();
        let target_name =
            package_binary_name(&request.package_name, version, configuration, &suffix);
        std::fs::copy(binary, package_dir.join(target_name))
            .map_err(|error| format!("Failed to copy {}: {error}", binary.display()))?;

        artifacts.push(BuildArtifact {
            version: Some(version.clone()),
            configuration: Some(configuration.clone()),
            kind: "version-package".to_string(),
            path: package_dir.display().to_string(),
        });

        if request.zip_enabled {
            let zip_path = create_zip_archive(&package_dir)?;
            artifacts.push(BuildArtifact {
                version: Some(version.clone()),
                configuration: Some(configuration.clone()),
                kind: "version-zip".to_string(),
                path: zip_path.display().to_string(),
            });
        }
    }

    Ok(artifacts)
}

fn copy_resources(plugin_root: &Path, binary: &Path, package_dir: &Path) -> Result<(), String> {
    let target = package_dir.join("res");
    if target.exists() {
        remove_path(&target)?;
    }

    if let Some(resource_dir) = find_resource_dir(plugin_root, binary) {
        copy_dir_recursive(&resource_dir, &target)
    } else {
        std::fs::create_dir_all(&target)
            .map_err(|error| format!("Failed to create {}: {error}", target.display()))
    }
}

fn find_resource_dir(plugin_root: &Path, binary: &Path) -> Option<PathBuf> {
    let direct = plugin_root.join("res");
    if direct.is_dir() {
        return Some(direct);
    }

    let built = binary.parent()?.join("res");
    if built.is_dir() {
        return Some(built);
    }

    find_nested_resource_dir(plugin_root)
}

fn find_nested_resource_dir(plugin_root: &Path) -> Option<PathBuf> {
    WalkDir::new(plugin_root)
        .min_depth(1)
        .max_depth(5)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_dir())
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| {
            path.file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name == "res")
        })
        .filter(|path| !path_contains_ignored_component(plugin_root, path))
        .find(|path| is_likely_module_resource_dir(path))
}

fn is_likely_module_resource_dir(resource_dir: &Path) -> bool {
    resource_dir.parent().is_some_and(|module_dir| {
        module_dir.join("project").is_dir() || module_dir.join("source").is_dir()
    })
}

fn path_contains_ignored_component(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root)
        .ok()
        .into_iter()
        .flat_map(|relative| relative.components())
        .filter_map(|component| component.as_os_str().to_str())
        .any(|name| matches!(name, ".git" | "build" | "dist" | "node_modules" | "target"))
}

fn package_folder_name(package_name: &str, version: &str, configuration: &str) -> String {
    format!(
        "{}_{}{}",
        package_name,
        package_version_label(version),
        configuration_suffix(configuration)
    )
}

fn package_binary_name(
    package_name: &str,
    version: &str,
    configuration: &str,
    extension: &str,
) -> String {
    format!(
        "{} {}{}{}",
        package_name,
        package_version_label(version),
        configuration_suffix(configuration),
        extension
    )
}

fn package_version_label(version: &str) -> &str {
    version.split_once('.').map_or(version, |(major, _)| major)
}

fn configuration_suffix(configuration: &str) -> &'static str {
    if configuration.eq_ignore_ascii_case("debug") {
        "_Debug"
    } else {
        ""
    }
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    for entry in WalkDir::new(source)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let relative = entry
            .path()
            .strip_prefix(source)
            .map_err(|error| error.to_string())?;
        let destination = target.join(relative);
        if entry.file_type().is_dir() {
            std::fs::create_dir_all(&destination)
                .map_err(|error| format!("Failed to create {}: {error}", destination.display()))?;
        } else {
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
            }
            std::fs::copy(entry.path(), &destination).map_err(|error| {
                format!(
                    "Failed to copy {} to {}: {error}",
                    entry.path().display(),
                    destination.display()
                )
            })?;
        }
    }
    Ok(())
}

pub fn remove_path(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }
    if path.is_file() {
        std::fs::remove_file(path)
            .map_err(|error| format!("Failed to remove {}: {error}", path.display()))
    } else {
        std::fs::remove_dir_all(path)
            .map_err(|error| format!("Failed to remove {}: {error}", path.display()))
    }
}

fn create_zip_archive(package_dir: &Path) -> Result<PathBuf, String> {
    let zip_name = format!(
        "{}.zip",
        package_dir
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| format!("Invalid package directory: {}", package_dir.display()))?
    );
    let zip_path = package_dir.with_file_name(zip_name);
    if zip_path.exists() {
        remove_path(&zip_path)?;
    }

    let file = File::create(&zip_path)
        .map_err(|error| format!("Failed to create {}: {error}", zip_path.display()))?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::<()>::default().compression_method(zip::CompressionMethod::Deflated);

    for entry in WalkDir::new(package_dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        let relative = path
            .strip_prefix(package_dir.parent().unwrap_or(package_dir))
            .map_err(|error| error.to_string())?;
        let name = relative.to_string_lossy().replace('\\', "/");
        if entry.file_type().is_dir() {
            if !name.is_empty() {
                zip.add_directory(name, options)
                    .map_err(|error| format!("Failed to add zip directory: {error}"))?;
            }
        } else {
            zip.start_file(name, options)
                .map_err(|error| format!("Failed to add zip file: {error}"))?;
            let mut input = File::open(path)
                .map_err(|error| format!("Failed to open {}: {error}", path.display()))?;
            let mut buffer = Vec::new();
            input
                .read_to_end(&mut buffer)
                .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
            zip.write_all(&buffer)
                .map_err(|error| format!("Failed to write zip data: {error}"))?;
        }
    }

    zip.finish()
        .map_err(|error| format!("Failed to finalize zip archive: {error}"))?;
    Ok(zip_path)
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    #[test]
    fn release_package_names_use_major_version_without_configuration() {
        assert_eq!(
            package_folder_name("Boghma WaterMark", "2024.4", "Release"),
            "Boghma WaterMark_2024"
        );
        assert_eq!(
            package_binary_name("Boghma WaterMark", "2024.4", "Release", ".xlib"),
            "Boghma WaterMark 2024.xlib"
        );
    }

    #[test]
    fn debug_package_names_keep_debug_suffix() {
        assert_eq!(
            package_folder_name("Boghma WaterMark", "2024.4", "Debug"),
            "Boghma WaterMark_2024_Debug"
        );
        assert_eq!(
            package_binary_name("Boghma WaterMark", "2026", "Debug", ".xlib"),
            "Boghma WaterMark 2026_Debug.xlib"
        );
    }

    #[test]
    fn per_version_package_copies_nested_module_resources() {
        let temp = TempTree::new("c4d-package-nested-res");
        let plugin = temp.path().join("BackHighlight");
        let module = plugin.join("draw.back");
        let binary_dir = temp.path().join("build").join("bin").join("Release");
        let binary = binary_dir.join("draw.back.xdl64");
        std::fs::create_dir_all(module.join("project")).unwrap();
        std::fs::create_dir_all(module.join("source")).unwrap();
        std::fs::create_dir_all(module.join("res").join("description")).unwrap();
        std::fs::create_dir_all(&binary_dir).unwrap();
        std::fs::write(module.join("res").join("c4d_symbols.h"), "symbols").unwrap();
        std::fs::write(
            module.join("res").join("description").join("drawback.res"),
            "CONTAINER",
        )
        .unwrap();
        std::fs::write(&binary, "binary").unwrap();

        let request = test_request(&plugin, temp.path().join("dist"));
        package_outputs(
            &request,
            &[("2025".to_string(), "Release".to_string(), binary)],
        )
        .unwrap();

        let package = temp.path().join("dist").join("BackHighlight_2025");
        assert!(package.join("BackHighlight 2025.xdl64").is_file());
        assert!(package.join("res").join("c4d_symbols.h").is_file());
        assert!(package
            .join("res")
            .join("description")
            .join("drawback.res")
            .is_file());
    }

    #[test]
    fn per_version_package_always_creates_resource_folder() {
        let temp = TempTree::new("c4d-package-empty-res");
        let plugin = temp.path().join("Plugin");
        let binary = temp.path().join("build").join("Plugin.xdl64");
        std::fs::create_dir_all(&plugin).unwrap();
        std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
        std::fs::write(&binary, "binary").unwrap();

        let request = test_request(&plugin, temp.path().join("dist"));
        package_outputs(
            &request,
            &[("2026".to_string(), "Release".to_string(), binary)],
        )
        .unwrap();

        assert!(temp
            .path()
            .join("dist")
            .join("BackHighlight_2026")
            .join("res")
            .is_dir());
    }

    fn test_request(plugin_root: &Path, output_dir: PathBuf) -> BuildRequest {
        BuildRequest {
            plugin_root: plugin_root.display().to_string(),
            module_name: "BackHighlight".to_string(),
            package_name: "BackHighlight".to_string(),
            versions: vec!["2026".to_string()],
            configuration: crate::types::BuildConfiguration::Release,
            sdk_source: crate::types::SdkSourceMode::ConfiguredThenInstalledThenOfficial,
            package_mode: PackageMode::PerVersion,
            zip_enabled: false,
            clean_output: true,
            refresh_sdk_cache: false,
            output_dir: Some(output_dir.display().to_string()),
        }
    }

    struct TempTree {
        path: PathBuf,
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
