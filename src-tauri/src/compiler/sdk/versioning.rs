//! SDK 版本号、归档名和下载地址规则。

use std::path::Path;

use super::MAXON_BASE_URL;

pub(super) fn sdk_download_candidates(version: &str) -> Vec<String> {
    let mut candidates = Vec::new();
    if let Some(archive) = known_sdk_archive_name(version) {
        candidates.push(format!("{MAXON_BASE_URL}/downloads/{archive}"));
    }
    candidates
}

pub(super) fn generated_download_url_if_known(version: &str) -> Option<String> {
    known_sdk_archive_name(version).map(|archive| format!("{MAXON_BASE_URL}/downloads/{archive}"))
}

pub(super) fn generated_download_url(version: &str) -> String {
    generated_download_url_if_known(version).unwrap_or_else(|| {
        format!(
            "{MAXON_BASE_URL}/downloads/Cinema_4D_CPP_SDK_{}_0_0.zip",
            version.replace('.', "_")
        )
    })
}

pub(super) fn sdk_archive_name(version: &str) -> String {
    known_sdk_archive_name(version)
        .map(str::to_string)
        .unwrap_or_else(|| format!("Cinema_4D_CPP_SDK_{}_0_0.zip", version.replace('.', "_")))
}

pub(super) fn known_sdk_archive_name(version: &str) -> Option<&'static str> {
    match version {
        "2024" | "2024.4" => Some("Cinema_4D_CPP_SDK_2024_4_0.zip"),
        "2025" => Some("Cinema_4D_CPP_SDK_2025_0_1.zip"),
        "2026" => Some("Cinema_4D_CPP_SDK_2026_0_0.zip"),
        _ => None,
    }
}

pub(super) fn installed_c4d_sdk_version(version: &str) -> Option<String> {
    let major = version.split('.').next()?;
    match major {
        "2024" => Some("2024.4".to_string()),
        "2025" => Some("2025".to_string()),
        "2026" => Some("2026".to_string()),
        _ => None,
    }
}

pub(super) fn normalize_path_key(path: &str) -> String {
    path.replace('\\', "/").to_lowercase()
}

pub(super) fn path_matches_sdk_version(path: &Path, version: &str) -> bool {
    let path_text = normalize_path_key(&path.display().to_string());
    sdk_version_match_tokens(version)
        .iter()
        .any(|token| path_text.contains(token))
}

pub(super) fn sdk_version_match_tokens(version: &str) -> Vec<String> {
    let major = version.split('.').next().unwrap_or(version);
    let mut tokens = vec![
        version.to_lowercase(),
        version.replace('.', "_").to_lowercase(),
        version.replace('.', "-").to_lowercase(),
    ];

    if version == major {
        tokens.push(format!("{major}_0_0").to_lowercase());
        tokens.push(format!("{major}-0-0").to_lowercase());
        tokens.push(format!("cinema_4d_cpp_sdk_{major}").to_lowercase());
        tokens.push(format!("cinema 4d {major}").to_lowercase());
    } else {
        tokens.push(format!("cinema_4d_cpp_sdk_{}", version.replace('.', "_")).to_lowercase());
        tokens.push(format!("cinema 4d {major}").to_lowercase());
    }

    tokens.sort();
    tokens.dedup();
    tokens
}

pub(super) fn version_folder(version: &str) -> String {
    version.replace('.', "_")
}

pub(super) fn version_sort_key(version: &str) -> u32 {
    let version = version.trim();
    if version.len() >= 2 && matches!(version.as_bytes()[0], b'R' | b'r') {
        return version[1..]
            .trim()
            .parse::<u32>()
            .map(|value| (2000 + value) * 100)
            .unwrap_or_default();
    }
    if version.len() >= 2 && matches!(version.as_bytes()[0], b'S' | b's') {
        return version[1..]
            .trim()
            .parse::<u32>()
            .map(|value| (2000 + value) * 100 + 1)
            .unwrap_or_default();
    }

    let mut parts = version.split('.');
    let major = parts
        .next()
        .and_then(|item| item.parse::<u32>().ok())
        .unwrap_or_default();
    let minor = parts
        .next()
        .and_then(|item| item.parse::<u32>().ok())
        .unwrap_or_default();
    major * 100 + minor
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn version_tokens_match_minor_sdk_names() {
        let tokens = sdk_version_match_tokens("2024.4");

        assert!(tokens.iter().any(|token| token == "2024.4"));
        assert!(tokens.iter().any(|token| token == "2024_4"));
        assert!(tokens
            .iter()
            .any(|token| token == "cinema_4d_cpp_sdk_2024_4"));
        assert!(path_matches_sdk_version(
            Path::new("/sdk/Cinema_4D_CPP_SDK_2024_4_0"),
            "2024.4"
        ));
    }
}
