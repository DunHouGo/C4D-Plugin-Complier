//! 打包目录的 zip 归档生成。

use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;
use zip::write::FileOptions;

use super::fs::remove_path;

pub(super) fn create_zip_archive(package_dir: &Path) -> Result<PathBuf, String> {
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
