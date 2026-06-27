//! 已编译 Cinema 4D 插件二进制文件的打包入口。

use std::path::{Path, PathBuf};

use crate::compiler::require_dir;
use crate::types::{BuildArtifact, BuildRequest, PackageMode};

mod archive;
mod fs;
mod naming;
mod resources;

use archive::create_zip_archive;
use fs::remove_path;
use naming::{package_binary_name, package_folder_name};
use resources::{copy_plugin_lib_directories, copy_resources};

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
        copy_plugin_lib_directories(plugin_root, &package_dir)?;
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

fn packaged_binary_name(binary: &Path) -> Result<PathBuf, String> {
    binary
        .file_name()
        .map(PathBuf::from)
        .ok_or_else(|| format!("Invalid built binary path: {}", binary.display()))
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
        copy_plugin_lib_directories(Path::new(&request.plugin_root), &package_dir)?;

        let target_name = packaged_binary_name(binary)?;
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

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
        assert!(package.join("draw.back.xdl64").is_file());
        assert!(package.join("res").join("c4d_symbols.h").is_file());
        assert!(package
            .join("res")
            .join("description")
            .join("drawback.res")
            .is_file());
    }

    #[test]
    fn per_version_package_copies_plugin_lib_directories() {
        let temp = TempTree::new("c4d-package-runtime");
        let plugin = temp.path().join("Plugin");
        let binary = temp.path().join("build").join("postwatermark.xdl64");
        let libs = plugin.join("libs").join("shared");
        std::fs::create_dir_all(&libs).unwrap();
        std::fs::create_dir_all(binary.parent().unwrap()).unwrap();
        std::fs::write(libs.join("helper.txt"), "runtime").unwrap();
        std::fs::write(&binary, "binary").unwrap();

        let request = test_request(&plugin, temp.path().join("dist"));
        package_outputs(
            &request,
            &[("2026".to_string(), "Release".to_string(), binary)],
        )
        .unwrap();

        let package = temp.path().join("dist").join("BackHighlight_2026");
        assert!(package.join("postwatermark.xdl64").is_file());
        assert!(package.join("libs").join("shared").join("helper.txt").is_file());
    }

    #[test]
    fn merged_package_keeps_binaries_in_single_folder() {
        let temp = TempTree::new("c4d-package-merged");
        let plugin = temp.path().join("Plugin");
        let binary_2025 = temp
            .path()
            .join("build")
            .join("2025")
            .join("postwatermark.xdl64");
        let binary_2026 = temp
            .path()
            .join("build")
            .join("2026")
            .join("postwatermark.xdl64");
        std::fs::create_dir_all(&plugin).unwrap();
        std::fs::create_dir_all(binary_2025.parent().unwrap()).unwrap();
        std::fs::create_dir_all(binary_2026.parent().unwrap()).unwrap();
        std::fs::write(&binary_2025, "binary-2025").unwrap();
        std::fs::write(&binary_2026, "binary-2026").unwrap();

        let mut request = test_request(&plugin, temp.path().join("dist"));
        request.package_mode = PackageMode::Merged;
        package_outputs(
            &request,
            &[
                ("2025".to_string(), "Release".to_string(), binary_2025),
                ("2026".to_string(), "Release".to_string(), binary_2026),
            ],
        )
        .unwrap();

        let package = temp.path().join("dist").join("BackHighlight");
        assert!(package.join("postwatermark 2025.xdl64").is_file());
        assert!(package.join("postwatermark 2026.xdl64").is_file());
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
