// Sidecar process management.
//
// `start_sidecar` looks for the PyInstaller'd `ghostloop-server`
// binary configured under tauri.conf.json bundle.resources. When the
// sidecar isn't bundled (dev loop, no PyInstaller pass yet), it falls
// back to `python3 -m ghostloop.mcp_server` so the app still boots.

use tauri::Emitter;
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

#[tauri::command]
pub async fn start_sidecar(app: tauri::AppHandle) -> Result<String, String> {
    let cmd = match app.shell().sidecar("ghostloop-server") {
        Ok(cmd) => cmd,
        Err(_) => app
            .shell()
            .command("python3")
            .args(["-m", "ghostloop.mcp_server"]),
    };

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
pub async fn intervention_emergency_stop(app: tauri::AppHandle) -> Result<(), String> {
    // Re-emit a frontend-side event so the embedded Next.js app POSTs
    // to its own /api/backend/v1/intervention/emergency_stop endpoint.
    // Keeping auth + URL knowledge inside the Next.js app means this
    // Tauri shell stays portable across deployment configurations.
    app.emit("intervention:emergency_stop", ())
        .map_err(|e| e.to_string())
}
