//! 发布包文件夹和二进制文件命名规则。

pub(super) fn package_folder_name(
    package_name: &str,
    version: &str,
    configuration: &str,
) -> String {
    format!(
        "{}_{}{}",
        package_name,
        package_version_label(version),
        configuration_suffix(configuration)
    )
}

pub(super) fn package_binary_name(
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

#[cfg(test)]
mod tests {
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
}
