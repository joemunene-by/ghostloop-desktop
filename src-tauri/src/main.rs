// Ghostloop desktop entry point.
//
// Two responsibilities at app start:
//   1. Spawn the sidecar Python runtime (`ghostloop-server`) so the
//      bundled Next.js frontend has a backend to talk to. The sidecar
//      lives in `binaries/ghostloop-server` (built from the Python
//      project; see scripts/build-sidecar.sh in this repo).
//   2. Start a gamepad-listener thread that emits JSON events over a
//      Tauri event channel (`gamepad`) so the frontend can react.
//
// Hot-key + tray menu wiring is in lib.rs; this file is tiny.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    ghostloop_desktop_lib::run();
}
