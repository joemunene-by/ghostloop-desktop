# Changelog

All notable changes to ghostloop-desktop. Versioning follows SemVer.

## [0.2.0] - 2026-05-11

### Added
- Voice control via the embedded WebView's Web Speech API on Windows
  (WebView2) and Linux (webkit2gtk). Wake phrase "ghostloop" maps to
  the safety + flight + locomotion primitives. macOS shows a "use
  system dictation" fallback for now; v0.3 swaps in native
  whisper.cpp.
- `voice_state` Tauri command so the frontend can render the right
  banner per platform.
- Profile-aware gamepad mapper in `ghostloop-ui/src/lib/gamepad.ts`.
  Drone, mobile base, quadruped, arm, humanoid each get a tailored
  stick / button layout. Drone profile takes the standard
  Mode-2-flight-stick convention (throttle on left-Y, yaw on left-X,
  pitch on right-Y, roll on right-X).
- Gamepad rumble for safety events. `rumble_pulse` Tauri command
  triggers a strong + weak motor pulse with caller-supplied intensity
  and duration. Wired up frontend-side for geofence block, force-cap
  trip, HITL escalation, and emergency stop.
- Native OS notifications via `tauri-plugin-notification`. The
  WebView calls `notify_alarm` whenever a new alarm appears in the
  backend feed; the OS surfaces it as a toast / banner / libnotify
  popup with severity-tagged title.
- GitHub Actions `ci.yml`: `cargo fmt --check`, `cargo clippy
  -D warnings`, `cargo check`, `cargo test`. Clippy and check run in
  parallel on macOS, Ubuntu, and Windows.
- GitHub Actions `release.yml`: cross-platform bundle scaffold (manual
  trigger only for now). Wiring in place to check out the ghostloop-ui
  sibling, build the frontend, and call `tauri-apps/tauri-action` on
  each OS. Auto-trigger on tag push is disabled until two pre-existing
  Tauri-with-Next.js bundling issues are fixed: (a) the
  `beforeBuildCommand` backgrounds `npm run start` with `&` and never
  exits, and (b) `frontendDist` points at `.next/static` rather than a
  static-export `out/` directory, so the WebView would have no page
  HTML to load. Both need an architectural pass that swaps the UI to
  `output: 'export'` and routes the desktop app's API calls directly
  to the sidecar Python runtime on `localhost:8000` instead of the
  Next.js proxy. Tracked for v0.2.1.
- Documentation expanded: gamepad support matrix (Xbox / PS5 /
  8BitDo / Stadia, wired and Bluetooth), drone control mapping, voice
  command table, native-input wiring example.

### Changed
- `src-tauri/src/lib.rs` split into four modules: `sidecar`,
  `gamepad`, `notification`, `voice`. The top-level `lib.rs` is now
  just the Tauri builder wiring.
- `Cargo.toml` adds `tauri-plugin-notification = "2.0"` and enables
  the `serde-serialize` feature on `gilrs` so force feedback is
  reachable from the rumble command.
- `tauri.conf.json` registers the notification plugin and bumps
  `version` to 0.2.0.
- `capabilities/main.json` grants `notification:default` so the
  WebView can call the notify command.

### Notes
- Bluetooth pairing is handled by the OS, not the app. Once paired
  via System Settings (macOS) / Bluetooth Devices (Windows) /
  bluetoothctl (Linux), the controller shows up alongside wired
  devices in the startup inventory log.
- Voice command list lives in the frontend so users can extend it
  without touching Rust.

## [0.1.0] - 2026-05-10

### Added
- Initial Tauri 2 shell wrapping the ghostloop-ui Next.js frontend.
- Sidecar Python runtime via PyInstaller, serving the production
  dashboard on `127.0.0.1:8000` with SQLite at `~/.ghostloop/store.db`.
- Native gamepad input via gilrs (~120 Hz polling).
- System-tray integration.
- Global hotkey for emergency stop.
