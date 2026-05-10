// Native OS notifications.
//
// The WebView calls `notify_alarm` whenever it picks up a new alarm
// from the backend (info / warn / error). The Rust side maps to the
// platform's native toast / banner / libnotify path via the official
// tauri-plugin-notification.
//
// Permission is requested lazily on first call; the user gets a
// system prompt the first time an alarm fires. After that we cache
// the granted state so we don't re-prompt.

use serde::Deserialize;
use tauri_plugin_notification::{NotificationExt, PermissionState};

#[derive(Debug, Deserialize)]
pub struct AlarmNotification {
    title: String,
    body: String,
    /// One of "info" | "warn" | "error". Used to pick an icon hint
    /// where the platform supports it.
    #[serde(default)]
    severity: String,
}

#[tauri::command]
pub async fn notify_alarm(
    app: tauri::AppHandle,
    notif: AlarmNotification,
) -> Result<(), String> {
    let notifier = app.notification();

    match notifier.permission_state() {
        Ok(PermissionState::Granted) => {}
        _ => match notifier.request_permission() {
            Ok(PermissionState::Granted) => {}
            Ok(state) => {
                return Err(format!("notification permission: {:?}", state));
            }
            Err(e) => return Err(format!("request permission failed: {}", e)),
        },
    }

    let title = format!(
        "{} {}",
        severity_glyph(&notif.severity).unwrap_or(""),
        notif.title
    )
    .trim()
    .to_string();

    notifier
        .builder()
        .title(title)
        .body(notif.body)
        .show()
        .map_err(|e| format!("show failed: {}", e))?;
    Ok(())
}

/// Returns an ASCII tag for the severity. The frontend already shows
/// colored icons in-app; on the OS side we keep this lightweight so
/// it works on every platform without bundling extra assets.
fn severity_glyph(severity: &str) -> Option<&'static str> {
    match severity {
        "error" => Some("[ERROR]"),
        "warn" => Some("[WARN]"),
        "info" => Some("[INFO]"),
        _ => None,
    }
}
