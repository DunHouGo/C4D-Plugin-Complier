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

    copy_resources(Path::new(&request.plugin_root), &package_dir)?;
    for (version, configuration, binary) in built_binaries {
        let suffix = binary
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!(".{value}"))
            .unwrap_or_default();
        let target_name = format!(
            "{} {} {}{}",
            request.package_name, version, configuration, suffix
        );
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
        let package_dir = output_root.join(format!(
            "{}_{}_{}",
            request.package_name, version, configuration
        ));
        if request.clean_output && package_dir.exists() {
            remove_path(&package_dir)?;
        }
        std::fs::create_dir_all(&package_dir)
            .map_err(|error| format!("Failed to create {}: {error}", package_dir.display()))?;
        copy_resources(Path::new(&request.plugin_root), &package_dir)?;

        let suffix = binary
            .extension()
            .and_then(|value| value.to_str())
            .map(|value| format!(".{value}"))
            .unwrap_or_default();
        let target_name = format!("{} {}{}", request.package_name, version, suffix);
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

fn copy_resources(plugin_root: &Path, package_dir: &Path) -> Result<(), String> {
    let resource_dir = plugin_root.join("res");
    if !resource_dir.is_dir() {
        return Ok(());
    }

    let target = package_dir.join("res");
    if target.exists() {
        remove_path(&target)?;
    }
    copy_dir_recursive(&resource_dir, &target)
}

fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
    for entry in WalkDir::new(source).into_iter().filter_map(Result::ok) {
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
