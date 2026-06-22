//! 原生通知命令。
//!
//! 基于 Tauri 通知插件提供跨平台原生通知。

use tauri::AppHandle;

/// 发送一条系统原生通知。
/// 移动端暂不支持通知，因此会返回错误。
#[tauri::command]
#[specta::specta]
pub async fn send_native_notification(
    app: AppHandle,
    title: String,
    body: Option<String>,
) -> Result<(), String> {
    log::info!("Sending native notification: {title}");

    #[cfg(not(mobile))]
    {
        use tauri_plugin_notification::NotificationExt;

        let mut notification = app.notification().builder().title(title);

        if let Some(body_text) = body {
            notification = notification.body(body_text);
        }

        match notification.show() {
            Ok(_) => {
                log::info!("Native notification sent successfully");
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to send native notification: {e}");
                Err(format!("Failed to send notification: {e}"))
            }
        }
    }

    #[cfg(mobile)]
    {
        let _ = (app, body);
        log::warn!("Native notifications not supported on mobile");
        Err("Native notifications not supported on mobile".to_string())
    }
}
