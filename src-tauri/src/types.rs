//! Tauri 应用共享类型和校验函数。

use regex::Regex;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::LazyLock;

/// 恢复数据文件最大体积 10MB。
pub const MAX_RECOVERY_DATA_BYTES: u32 = 10_485_760;

/// 用于文件名校验的预编译正则。
/// 仅允许字母数字、短横线、下划线和一个扩展名。
pub static FILENAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9_-]+(\.[a-zA-Z0-9]+)?$")
        .expect("Failed to compile filename regex pattern")
});

// ============================================================================
// 偏好设置
// ============================================================================

/// 会持久化到磁盘的应用偏好设置。
/// 仅包含跨会话需要保留的设置。
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppPreferences {
    pub theme: String,
    /// 用户偏好的界面语言，例如 `zh-CN` 或 `en-US`。
    /// 为 None 时根据系统语言自动选择。
    pub language: Option<String>,
}

// ============================================================================
// C4D 插件编译器
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum BuildConfiguration {
    Debug,
    Release,
    Both,
}

impl BuildConfiguration {
    pub fn cmake_configs(&self) -> Vec<&'static str> {
        match self {
            Self::Debug => vec!["Debug"],
            Self::Release => vec!["Release"],
            Self::Both => vec!["Debug", "Release"],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum PackageMode {
    Merged,
    PerVersion,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum CompilerPlatform {
    Windows,
    Macos,
    Linux,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum SdkSourceMode {
    ConfiguredThenInstalledThenOfficial,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BuildRequest {
    pub plugin_root: String,
    pub module_name: String,
    pub package_name: String,
    pub versions: Vec<String>,
    pub configuration: BuildConfiguration,
    pub sdk_source: SdkSourceMode,
    pub package_mode: PackageMode,
    pub zip_enabled: bool,
    pub clean_output: bool,
    pub refresh_sdk_cache: bool,
    pub output_dir: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BuildJobId {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct ToolStatus {
    pub found: bool,
    pub path: Option<String>,
    pub version: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct EnvironmentReport {
    pub os: String,
    pub supported: bool,
    pub compiler_platform: CompilerPlatform,
    pub cmake_preset: Option<String>,
    pub binary_extension: Option<String>,
    pub cmake: ToolStatus,
    pub visual_studio: ToolStatus,
    pub windows_sdk: ToolStatus,
    pub xcode: ToolStatus,
    pub clang: ToolStatus,
    pub python: ToolStatus,
    pub installed_sdk_zips: Vec<InstalledSdkZip>,
    pub installed_c4d_versions: Vec<InstalledC4dVersion>,
    pub cache_root: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InstalledSdkZip {
    pub version: String,
    pub path: String,
    pub size_bytes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct InstalledC4dVersion {
    pub version: String,
    pub path: String,
    pub sdk_version: String,
    pub download_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum SdkResolutionSource {
    Config,
    InstalledZip,
    OfficialDownload,
    Cache,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SdkResolution {
    pub version: String,
    pub source: SdkResolutionSource,
    pub sdk_root: Option<String>,
    pub archive_path: Option<String>,
    pub download_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SdkVersionOption {
    pub version: String,
    pub label: String,
    pub configured: bool,
    pub sdk_root: Option<String>,
    pub sdk_zip: Option<String>,
    pub download_url: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BuildArtifact {
    pub version: Option<String>,
    pub configuration: Option<String>,
    pub kind: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BuildLogEvent {
    pub job_id: String,
    pub level: String,
    pub category: String,
    pub timestamp: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BuildProgressEvent {
    pub job_id: String,
    pub current: u32,
    pub total: u32,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct BuildFinishedEvent {
    pub job_id: String,
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SdkRootConfig {
    pub sdk_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SdkSourceOverride {
    pub sdk_root: Option<String>,
    pub sdk_zip: Option<String>,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SdkSourceConfig {
    pub sdk_root: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SdkAutoConfigReport {
    pub sdk_root: Option<String>,
    pub installed_versions: Vec<InstalledC4dVersion>,
    pub versions: Vec<SdkVersionOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub enum SetupRequirementStatus {
    Ready,
    Warning,
    Missing,
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SetupRequirement {
    pub key: String,
    pub label: String,
    pub status: SetupRequirementStatus,
    pub detail: String,
    pub path: Option<String>,
    pub version: Option<String>,
    pub auto_installable: bool,
    pub install_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct SdkSetupReport {
    pub sdk_root: Option<String>,
    pub installed_versions: Vec<InstalledC4dVersion>,
    pub versions: Vec<SdkVersionOption>,
    pub prepared_versions: Vec<SdkResolution>,
    pub requirements: Vec<SetupRequirement>,
    pub summary: String,
}

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: None, // None 表示跟随系统语言。
        }
    }
}

// ============================================================================
// 恢复错误
// ============================================================================

/// 恢复操作错误类型，供前端按类型匹配。
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum RecoveryError {
    /// 文件不存在，这是预期情况，不代表系统故障。
    FileNotFound,
    /// 文件名校验失败。
    ValidationError { message: String },
    /// 数据超过大小限制。
    DataTooLarge { max_bytes: u32 },
    /// 文件系统读写错误。
    IoError { message: String },
    /// JSON 序列化或反序列化错误。
    ParseError { message: String },
}

impl std::fmt::Display for RecoveryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecoveryError::FileNotFound => write!(f, "File not found"),
            RecoveryError::ValidationError { message } => write!(f, "Validation error: {message}"),
            RecoveryError::DataTooLarge { max_bytes } => {
                write!(f, "Data too large (max {max_bytes} bytes)")
            }
            RecoveryError::IoError { message } => write!(f, "IO error: {message}"),
            RecoveryError::ParseError { message } => write!(f, "Parse error: {message}"),
        }
    }
}

// ============================================================================
// 校验函数
// ============================================================================

/// 校验文件名是否适合安全文件系统操作。
/// 仅允许字母数字、短横线、下划线和一个扩展名。
pub fn validate_filename(filename: &str) -> Result<(), String> {
    if filename.is_empty() {
        return Err("Filename cannot be empty".to_string());
    }

    if filename.chars().count() > 100 {
        return Err("Filename too long (max 100 characters)".to_string());
    }

    if !FILENAME_PATTERN.is_match(filename) {
        return Err(
            "Invalid filename: only alphanumeric characters, dashes, underscores, and dots allowed"
                .to_string(),
        );
    }

    Ok(())
}

/// 按字符数而非字节数校验字符串长度。
pub fn validate_string_input(input: &str, max_len: usize, field_name: &str) -> Result<(), String> {
    let char_count = input.chars().count();
    if char_count > max_len {
        return Err(format!("{field_name} too long (max {max_len} characters)"));
    }
    Ok(())
}

/// 校验主题取值。
pub fn validate_theme(theme: &str) -> Result<(), String> {
    match theme {
        "light" | "dark" | "system" => Ok(()),
        _ => Err("Invalid theme: must be 'light', 'dark', or 'system'".to_string()),
    }
}
