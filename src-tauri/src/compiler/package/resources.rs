//! 输出插件目录的 res 资源查找和复制。

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use super::fs::{copy_dir_recursive, remove_path};

pub(super) fn copy_resources(
    plugin_root: &Path,
    binary: &Path,
    package_dir: &Path,
) -> Result<(), String> {
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

pub(super) fn copy_plugin_lib_directories(
    plugin_root: &Path,
    package_dir: &Path,
) -> Result<(), String> {
    for directory_name in ["libs"] {
        let source = plugin_root.join(directory_name);
        if !source.is_dir() {
            continue;
        }

        let target = package_dir.join(directory_name);
        if target.exists() {
            remove_path(&target)?;
        }
        copy_dir_recursive(&source, &target)?;
    }

    Ok(())
}

fn find_resource_dir(plugin_root: &Path, binary: &Path) -> Option<PathBuf> {
    let built = binary.parent()?.join("res");
    if built.is_dir() {
        return Some(built);
    }

    let direct = plugin_root.join("res");
    if direct.is_dir() {
        return Some(direct);
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
        .any(|name| {
            matches!(
                name,
                ".git" | "build" | "dist" | "dist-test-debug" | "node_modules" | "target"
            )
        })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::copy_resources;

    #[test]
    fn copy_resources_prefers_built_res_over_plugin_root_res() {
        let temp = TempTree::new("c4d-package-res-priority");
        let plugin_root = temp.path().join("Plugin");
        let binary = temp.path().join("build").join("bin").join("Release").join("Plugin.xdl64");
        let package_dir = temp.path().join("dist");
        let built_res = binary.parent().unwrap().join("res");
        let root_res = plugin_root.join("res");

        std::fs::create_dir_all(&built_res).unwrap();
        std::fs::create_dir_all(root_res.join("description")).unwrap();
        std::fs::create_dir_all(plugin_root.join("project")).unwrap();
        std::fs::create_dir_all(plugin_root.join("source")).unwrap();
        std::fs::create_dir_all(package_dir.join("res")).unwrap();
        std::fs::write(built_res.join("marker.txt"), "new").unwrap();
        std::fs::write(root_res.join("marker.txt"), "old").unwrap();

        copy_resources(&plugin_root, &binary, &package_dir).unwrap();

        assert!(package_dir.join("res").join("marker.txt").is_file());
        let contents = std::fs::read_to_string(package_dir.join("res").join("marker.txt")).unwrap();
        assert_eq!(contents, "new");
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
