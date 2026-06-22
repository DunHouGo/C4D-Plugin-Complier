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
