//! Tauri 应用库入口。
//!
//! 这里负责初始化 Tauri 应用。
//! 命令实现放在 `commands` 模块中，
//! 共享类型放在 `types` 模块中。

mod bindings;
mod commands;
mod compiler;
mod types;
mod utils;

use compiler::jobs::JobManager;
use tauri::Manager;

/// 应用入口，负责注册插件并初始化应用。
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = bindings::generate_bindings();

    // 调试构建时导出 TypeScript 绑定。
    #[cfg(debug_assertions)]
    if let Err(error) = bindings::export_ts_bindings() {
        log::warn!("{error}");
    }

    // 创建带通用插件的应用构建器。
    let mut app_builder = tauri::Builder::default();

    // 单实例插件必须最先注册。
    // 用户启动第二个实例时，聚焦已有主窗口。
    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
                let _ = window.unminimize();
            }
        }));
    }

    // 窗口状态插件用于保存和恢复窗口位置与尺寸。
    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(tauri_plugin_window_state::StateFlags::all())
                .build(),
        );
    }

    // 更新插件用于应用内检查更新。
    #[cfg(desktop)]
    {
        app_builder = app_builder.plugin(tauri_plugin_updater::Builder::new().build());
    }

    app_builder = app_builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                // 开发环境使用 Debug 等级，生产环境使用 Info 等级。
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .targets([
                    // 始终输出到 stdout，方便开发调试。
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    // 同步输出到 WebView 控制台。
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Webview),
                    // macOS 下写入系统日志，可在 Console.app 查看。
                    #[cfg(target_os = "macos")]
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: None,
                    }),
                ])
                .build(),
        );

    app_builder
        .manage(JobManager::default())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            log::info!("Application starting up");
            log::debug!(
                "App handle initialized for package: {}",
                app.package_info().name
            );

            // 注意：应用菜单由 JavaScript 创建，以复用前端国际化。
            // 菜单实现见 src/lib/menu.ts。

            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
