//! 处理平台差异的跨平台工具。
//!
//! 这些工具用于应用中的跨平台逻辑。
//! 并非每个工具都会在当前代码中直接使用。
//!
//! 本模块提供编写 Tauri 跨平台 Rust 代码的辅助函数。
//! 平台专用行为优先使用条件编译。
//!
//! # 示例
//!
//! ```ignore
//! use crate::utils::platform;
//!
//! // 将 Windows 路径标准化为前端使用的正斜杠。
//! let normalized = platform::normalize_path_for_serialization(&some_path);
//!
//! // 使用 cfg 编写平台专用逻辑。
//! #[cfg(target_os = "macos")]
//! fn macos_specific() {
//!     // 仅 macOS 使用的代码。
//! }
//!
//! #[cfg(target_os = "windows")]
//! fn windows_specific() {
//!     // 仅 Windows 使用的代码。
//! }
//!
//! #[cfg(target_os = "linux")]
//! fn linux_specific() {
//!     // 仅 Linux 使用的代码。
//! }
//! ```

// 允许未使用代码，因为这些工具供跨平台场景按需调用。
#![allow(dead_code)]

use std::path::Path;

/// 将路径标准化为正斜杠，便于前端一致处理。
///
/// 例如 Windows 路径 `C:\Users\foo\bar.txt` 会变成 `C:/Users/foo/bar.txt`。
/// 向 React 前端传递路径时很有用，
/// 因为前端统一按正斜杠处理。
///
/// macOS 和 Linux 路径本来就是正斜杠，
/// 因此这里主要用于保持跨平台一致性。
///
/// # 示例
///
/// ```ignore
/// use std::path::Path;
/// use crate::utils::platform::normalize_path_for_serialization;
///
/// let path = Path::new("some/path/file.txt");
/// let normalized = normalize_path_for_serialization(path);
/// assert_eq!(normalized, "some/path/file.txt");
/// ```
pub fn normalize_path_for_serialization(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

/// 当前运行在 macOS 时返回 true。
///
/// 运行时检查使用它，编译期分支请使用 `#[cfg(target_os = "macos")]`。
#[inline]
pub const fn is_macos() -> bool {
    cfg!(target_os = "macos")
}

/// 当前运行在 Windows 时返回 true。
///
/// 运行时检查使用它，编译期分支请使用 `#[cfg(target_os = "windows")]`。
#[inline]
pub const fn is_windows() -> bool {
    cfg!(target_os = "windows")
}

/// 当前运行在 Linux 时返回 true。
///
/// 运行时检查使用它，编译期分支请使用 `#[cfg(target_os = "linux")]`。
#[inline]
pub const fn is_linux() -> bool {
    cfg!(target_os = "linux")
}

/// 返回当前平台字符串：`macos`、`windows` 或 `linux`。
///
/// 需要向前端传递平台信息，
/// 且不想额外调用 OS 插件时可使用。
pub const fn current_platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "linux"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_normalize_path_forward_slashes() {
        let path = PathBuf::from("foo/bar/baz.txt");
        let normalized = normalize_path_for_serialization(&path);
        assert_eq!(normalized, "foo/bar/baz.txt");
    }

    #[test]
    fn test_normalize_path_empty() {
        let path = PathBuf::from("");
        let normalized = normalize_path_for_serialization(&path);
        assert_eq!(normalized, "");
    }

    #[test]
    fn test_current_platform_is_valid() {
        let platform = current_platform();
        assert!(
            platform == "macos" || platform == "windows" || platform == "linux",
            "Platform should be one of: macos, windows, linux"
        );
    }

    #[test]
    fn test_platform_detection_consistency() {
        // 三个平台判断中应该只有一个为 true。
        let platforms = [is_macos(), is_windows(), is_linux()];
        let count = platforms.iter().filter(|&&x| x).count();
        assert_eq!(count, 1, "Exactly one platform should be detected");
    }
}
