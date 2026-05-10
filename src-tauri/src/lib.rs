// Ghostloop desktop, Tauri shell.
//
// Surfaces exposed to the embedded Next.js WebView:
//
//   start_sidecar                  spawns the Python runtime
//                                  (ghostloop-server sidecar binary,
//                                  or `python3 -m ghostloop.mcp_server`
//                                  fallback for dev).
//   intervention_emergency_stop    bounces an `intervention:emergency_stop`
//                                  event to the WebView so the Next.js
//                                  side issues the POST.
//   notify_alarm                   raises a native OS notification
//                                  (toast on Windows, banner on
//                                  macOS, libnotify on Linux). Called
//                                  by the WebView whenever it picks
//                                  up a new alarm from the backend.
//   rumble_pulse                   triggers a short rumble pulse on
//                                  the active gamepad (used by the
//                                  WebView to signal safety events:
//                                  geofence block, force-cap trip,
//                                  HITL escalation, emergency stop).
//   voice_state                    reports whether voice support is
//                                  available on this platform (the
//                                  WebView decides the engine; this
//                                  function carries the metadata so
//                                  the UI can show the right banner).
//
// Plus two passive listeners:
//
//   gamepad listener thread        polls connected controllers via
//                                  gilrs at ~120 Hz, forwards axis +
//                                  button events to the WebView on
//                                  the `gamepad` event channel.
//   voice listener (frontend)      the WebView uses the Web Speech
//                                  API where available (WebView2 on
//                                  Windows, webkit2gtk on Linux) and
//                                  falls back to a "voice unavailable"
//                                  banner on macOS WKWebView. Native
//                                  STT lands in v0.3 via whisper-rs;
//                                  the plumbing here is engine-agnostic.

mod gamepad;
mod notification;
mod sidecar;
mod voice;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            gamepad::spawn_listener(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            sidecar::start_sidecar,
            sidecar::intervention_emergency_stop,
            notification::notify_alarm,
            gamepad::rumble_pulse,
            voice::voice_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
