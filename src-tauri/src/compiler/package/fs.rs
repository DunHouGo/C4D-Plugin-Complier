//! 打包阶段使用的文件复制和删除工具。

use std::path::Path;

use walkdir::WalkDir;

pub(super) fn copy_dir_recursive(source: &Path, target: &Path) -> Result<(), String> {
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

pub(super) fn remove_path(path: &Path) -> Result<(), String> {
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
