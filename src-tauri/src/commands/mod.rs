//! 按领域组织的 Tauri 命令处理模块。
//!
//! 每个子模块包含一组相关命令及辅助函数。
//! 需要具体命令时从对应子模块导入。

pub mod build_queue;
pub mod compiler;
pub mod diagnostics;
pub mod notifications;
pub mod preferences;
pub mod recovery;
