// Ghostloop desktop — Tauri shell.
//
// Three surfaces:
//   - `start_sidecar` Tauri command: spawns the Python runtime that
//     the frontend talks to (sidecar `binaries/ghostloop-server`).
//   - `gamepad` listener thread: polls connected controllers via
//     `gilrs` and forwards normalized events to the frontend over the
//     `gamepad` Tauri event channel.
//   - `intervention_emergency_stop` Tauri command: a shortcut for the
//     dashboard's POST /v1/intervention/emergency_stop, exposed so
//     the frontend can wire it to a global hot-key.
//
// Everything else (UI, navigation, fleet rendering) is the Next.js
// frontend embedded as the WebView.

use serde::Serialize;
use std::time::Duration;
use tauri::{Emitter, Manager};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Serialize, Clone)]
struct GamepadEvent {
    pad_id: usize,
    pad_name: String,
    kind: &'static str,
    code: String,
    value: f32,
}

#[tauri::command]
async fn start_sidecar(app: tauri::AppHandle) -> Result<String, String> {
    // Sidecar binary path is configured in tauri.conf.json's
    // `bundle.resources` once the Python build artifact is generated.
    // For now, fall back to invoking system `python3 -m ghostloop.mcp_server`
    // so the dev-loop works without a packaged sidecar.
    let cmd = app
        .shell()
        .sidecar("ghostloop-server")
        .or_else(|_| {
            app.shell()
                .command("python3")
                .args(["-m", "ghostloop.mcp_server"])
                .into_iter()
                .next()
                .ok_or("no python3 + ghostloop fallback available".to_string())
        })
        .map_err(|e| e.to_string())?;

    let (mut rx, _child) = cmd.spawn().map_err(|e| format!("spawn failed: {}", e))?;

    tauri::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let CommandEvent::Stdout(line) | CommandEvent::Stderr(line) = event {
                let line = String::from_utf8_lossy(&line).trim().to_string();
                if !line.is_empty() {
                    eprintln!("[sidecar] {}", line);
                }
            }
        }
    });

    Ok("sidecar spawned".into())
}

#[tauri::command]
async fn intervention_emergency_stop(
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Re-emit a frontend-side event so the embedded Next.js app POSTs
    // to its own /api/backend/v1/intervention/emergency_stop endpoint.
    // Keeping the auth + URL knowledge inside the Next.js app means
    // this Tauri shell stays portable across deployment configurations.
    app.emit("intervention:emergency_stop", ())
        .map_err(|e| e.to_string())
}

fn spawn_gamepad_listener(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mut gilrs = match gilrs::Gilrs::new() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("[gamepad] failed to init gilrs: {}", e);
                return;
            }
        };
        // Initial inventory log so the user knows what's connected.
        for (id, gp) in gilrs.gamepads() {
            eprintln!("[gamepad] connected #{}: {}", usize::from(id), gp.name());
        }
        loop {
            while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
                let pad = gilrs.gamepad(id);
                let payload = match event {
                    gilrs::EventType::ButtonPressed(btn, _) => GamepadEvent {
                        pad_id: usize::from(id),
                        pad_name: pad.name().to_string(),
                        kind: "button_press",
                        code: format!("{:?}", btn),
                        value: 1.0,
                    },
                    gilrs::EventType::ButtonReleased(btn, _) => GamepadEvent {
                        pad_id: usize::from(id),
                        pad_name: pad.name().to_string(),
                        kind: "button_release",
                        code: format!("{:?}", btn),
                        value: 0.0,
                    },
                    gilrs::EventType::AxisChanged(axis, value, _) => GamepadEvent {
                        pad_id: usize::from(id),
                        pad_name: pad.name().to_string(),
                        kind: "axis",
                        code: format!("{:?}", axis),
                        value,
                    },
                    _ => continue,
                };
                let _ = app.emit("gamepad", payload);
            }
            std::thread::sleep(Duration::from_millis(8)); // ~120 Hz polling
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            spawn_gamepad_listener(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_sidecar,
            intervention_emergency_stop,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
