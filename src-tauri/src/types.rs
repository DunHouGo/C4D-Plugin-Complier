//! Shared types and validation functions for the Tauri application.

use regex::Regex;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::LazyLock;

/// Maximum size for recovery data files (10MB)
pub const MAX_RECOVERY_DATA_BYTES: u32 = 10_485_760;

/// Pre-compiled regex pattern for filename validation.
/// Only allows alphanumeric characters, dashes, underscores, and a single extension.
pub static FILENAME_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[a-zA-Z0-9_-]+(\.[a-zA-Z0-9]+)?$")
        .expect("Failed to compile filename regex pattern")
});

// ============================================================================
// Preferences
// ============================================================================

/// Application preferences that persist to disk.
/// Only contains settings that should be saved between sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
pub struct AppPreferences {
    pub theme: String,
    /// User's preferred language (e.g., "en", "es", "de")
    /// If None, uses system locale detection
    pub language: Option<String>,
}

// ============================================================================
// C4D Plugin Compiler
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

impl Default for AppPreferences {
    fn default() -> Self {
        Self {
            theme: "system".to_string(),
            language: None, // None means use system locale
        }
    }
}

// ============================================================================
// Recovery Errors
// ============================================================================

/// Error types for recovery operations (typed for frontend matching)
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(tag = "type")]
pub enum RecoveryError {
    /// File does not exist (expected case, not a failure)
    FileNotFound,
    /// Filename validation failed
    ValidationError { message: String },
    /// Data exceeds size limit
    DataTooLarge { max_bytes: u32 },
    /// File system read/write error
    IoError { message: String },
    /// JSON serialization/deserialization error
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
// Validation Functions
// ============================================================================

/// Validates a filename for safe file system operations.
/// Only allows alphanumeric characters, dashes, underscores, and a single extension.
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

/// Validates string input length (by character count, not bytes).
pub fn validate_string_input(input: &str, max_len: usize, field_name: &str) -> Result<(), String> {
    let char_count = input.chars().count();
    if char_count > max_len {
        return Err(format!("{field_name} too long (max {max_len} characters)"));
    }
    Ok(())
}

/// Validates theme value.
pub fn validate_theme(theme: &str) -> Result<(), String> {
    match theme {
        "light" | "dark" | "system" => Ok(()),
        _ => Err("Invalid theme: must be 'light', 'dark', or 'system'".to_string()),
    }
}
