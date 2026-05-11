# ghostloop-desktop

Native desktop control panel for [ghostloop](https://github.com/joemunene-by/ghostloop). Tauri 2 shell wrapping the [ghostloop-ui](https://github.com/joemunene-by/ghostloop-ui) Next.js frontend, with native gamepad input and a sidecar Python runtime that serves the dashboard backend on first launch.

## What you get

- **Single-file desktop app** for macOS / Windows / Linux. No `pip install`, no `npm run dev`: double-click to run.
- **Sidecar Python runtime** packaged via PyInstaller. Serves the ghostloop production dashboard on `127.0.0.1:8000` with a SQLite store at `~/.ghostloop/store.db`.
- **Native gamepad input (wired + Bluetooth)** via [gilrs](https://gitlab.com/gilrs-project/gilrs). Profile-aware mapper translates stick / button / trigger events into the right primitives for whatever robot you select. Drone, mobile base, quadruped, arm, humanoid: all covered.
- **Voice control** via the embedded WebView's Web Speech API on Windows and Linux. macOS gets a "use system dictation" fallback in v0.2; v0.3 swaps in native whisper.cpp for parity.
- **Gamepad rumble for safety events** so an operator feels a geofence block, force-cap trip, HITL escalation, or emergency stop before they see it on screen.
- **Native OS notifications** (Windows toast / macOS banner / Linux libnotify) for new alarms, so you can step away from the screen and still get paged.
- **System-tray integration** so the app stays accessible across desktops.
- **Hot-key for emergency stop** wired to `intervention_emergency_stop`.

## Gamepad support

`gilrs` talks the OS HID stack, so anything the operating system sees as a gamepad just works: Xbox, PlayStation, 8BitDo, Stadia, generic HID. Wired USB or paired over Bluetooth, no difference to the app.

Tested controllers:

| Controller            | macOS | Windows | Linux |
|-----------------------|:-----:|:-------:|:-----:|
| Xbox Series X (USB)   |  yes  |   yes   |  yes  |
| Xbox Series X (BT)    |  yes  |   yes   |  yes  |
| PS5 DualSense (USB)   |  yes  |   yes   |  yes  |
| PS5 DualSense (BT)    |  yes  |   yes   |  yes  |
| 8BitDo Pro 2 (USB+BT) |  yes  |   yes   |  yes  |
| Stadia (USB+BT)       |  yes  |   yes   |  yes  |

Pair via Bluetooth the same way you would for any other controller (System Settings on macOS, Bluetooth Devices on Windows, `bluetoothctl pair <mac>` on Linux). Once paired, launch ghostloop-desktop and the controller shows up in the gamepad inventory log on startup.

### Drone control mapping

Pick the **`tello`** profile (or any drone profile) and the gamepad maps to flight controls automatically:

| Input                | Action                                  |
|----------------------|-----------------------------------------|
| Left stick Y         | Throttle (climb / descend)              |
| Left stick X         | Yaw (rotate)                            |
| Right stick Y        | Pitch (forward / backward)              |
| Right stick X        | Roll (strafe left / right)              |
| A (South)            | Takeoff                                 |
| B (East)             | Land                                    |
| Y (North)            | Emergency stop                          |
| Right trigger        | Fine altitude up                        |
| Left trigger         | Fine altitude down                      |

Other robot classes (mobile base, quadruped, arm, humanoid) get their own auto-applied mapping. See `src/lib/gamepad.ts` in the ghostloop-ui repo for the full table.

## Voice control

In v0.2 the embedded WebView handles speech-to-text via the Web Speech API. That covers Windows (WebView2 = Chromium) and Linux (webkit2gtk with the speech flag). macOS WKWebView doesn't ship a SpeechRecognition implementation, so on macOS today voice control shows a "use system dictation" prompt; v0.3 bundles whisper.cpp so every platform has the same hands-free experience.

Default wake phrase: **"ghostloop"**. Commands the recognizer maps out of the box:

| Phrase                       | Action                          |
|------------------------------|---------------------------------|
| "ghostloop stop" / "halt"    | Emergency stop                  |
| "ghostloop pause"            | Intervention pause              |
| "ghostloop resume"           | Intervention resume             |
| "ghostloop takeoff"          | Drone takeoff                   |
| "ghostloop land"             | Drone land                      |
| "ghostloop move forward"     | drive(linear_x=0.2)             |
| "ghostloop turn left"        | drive(angular_z=0.5)            |
| "ghostloop wave"             | Humanoid wave                   |

The list lives in the frontend, so you can extend it without touching the Rust shell.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Tauri (Rust shell)                                     │
│  ├── WebView (Next.js: ghostloop-ui)                    │
│  │   ├── Web Speech API listener (voice in)             │
│  │   └── Gamepad event consumer (Tauri channel)         │
│  ├── Sidecar process (Python: ghostloop production      │
│  │   dashboard via PyInstaller'd binary)                │
│  └── Native event bridge:                               │
│      ├── gamepad listener (gilrs, 120 Hz)               │
│      ├── rumble_pulse command (force feedback out)      │
│      ├── notify_alarm command (OS toast / banner)       │
│      ├── voice_state command (platform capability)      │
│      └── intervention_emergency_stop command (hotkey)   │
└─────────────────────────────────────────────────────────┘
                  │
                  ▼  HTTP (localhost:8000)
        ┌──────────────────────┐
        │  ghostloop runtime   │
        │  + safety pipeline   │
        │  + SQLite store      │
        └──────────────────────┘
```

## Repo layout

```
ghostloop-desktop/
├── .github/workflows/
│   ├── ci.yml                Rust lint + clippy + check + test (PRs)
│   └── release.yml           Cross-platform bundle on tag push
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── capabilities/main.json
│   └── src/
│       ├── main.rs           process entry
│       ├── lib.rs            module wiring
│       ├── sidecar.rs        Python runtime spawn + e-stop bridge
│       ├── gamepad.rs        gilrs listener + rumble_pulse command
│       ├── notification.rs   notify_alarm command
│       └── voice.rs          voice_state command (engine-agnostic plumbing)
├── scripts/
│   └── build-sidecar.sh      PyInstaller -> ghostloop-server-<triple>
├── package.json
└── README.md
```

The frontend (Next.js) is consumed from a sibling clone of [ghostloop-ui](https://github.com/joemunene-by/ghostloop-ui), referenced via `tauri.conf.json`'s `beforeDevCommand` and `frontendDist`.

## Build (one-time setup)

Prerequisites: Rust (stable), Node 20+, Python 3.10+.

On Linux you also need:

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev \
  librsvg2-dev patchelf libudev-dev libasound2-dev
```

Then:

```bash
git clone https://github.com/joemunene-by/ghostloop-desktop
git clone https://github.com/joemunene-by/ghostloop-ui ../ghostloop-ui

cd ghostloop-desktop
npm install
cd ../ghostloop-ui && npm install && cd ../ghostloop-desktop

# Build the sidecar binary (one-time per machine).
./scripts/build-sidecar.sh
# -> src-tauri/binaries/ghostloop-server-<rustc-triple>

# Dev loop: opens the desktop window with hot-reloading frontend.
npm run dev

# Production bundle (.dmg / .AppImage / .deb / .msi / .nsis):
npm run build
```

The bundle artefacts land in `src-tauri/target/release/bundle/`.

## Continuous integration

`ci.yml` runs on every PR and push to main: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo check` and `cargo test`. The clippy and check steps run on macOS, Ubuntu, and Windows in parallel so platform-specific regressions surface immediately. Each job uses `Swatinem/rust-cache` so warm runs land in ~90 seconds.

`release.yml` is wired up for cross-platform bundling via `tauri-apps/tauri-action@v0` on macOS / Linux / Windows but is currently set to **workflow_dispatch only**. The auto-trigger on tag push is parked behind a v0.2.1 fix because the existing `tauri.conf.json` backgrounds `npm run start` from `beforeBuildCommand` (which never returns) and points `frontendDist` at `.next/static` instead of a Next.js static-export `out/` directory. Once the UI is reconfigured with `output: 'export'` and the desktop app routes its API calls directly to the sidecar's `localhost:8000`, the auto-trigger comes back. Until then, code lands on tags but DMG / MSI / AppImage artefacts have to be built locally with `npm run build`.

## Native input details

`gilrs` polls every connected gamepad at ~120 Hz. Each `Event` is normalised into:

```ts
{ pad_id: number, pad_name: string, kind: "button_press" | "button_release" | "axis", code: string, value: number }
```

The frontend listens via `@tauri-apps/api/event`:

```ts
import { listen } from "@tauri-apps/api/event"
import { applyEvent, blankState, dispatchFor, classOf } from "@/lib/gamepad"

const state = blankState()
const cls = classOf(activeProfile)   // "drone" | "mobile_base" | ...

await listen<GamepadEvent>("gamepad", (e) => {
  applyEvent(state, e.payload)
  const intent = dispatchFor(cls, state)
  if (intent) backend.dispatch(intent)  // POST /api/backend/v1/runtime/step
})
```

Rumble back through the Tauri bridge:

```ts
import { invoke } from "@tauri-apps/api/core"

// On geofence block / force-cap trip / HITL escalation:
await invoke("rumble_pulse", {
  req: { intensity: 0.7, duration_ms: 250 },
})
```

## Roadmap

- **v0.1** shell + sidecar + gamepad + tray + e-stop hotkey
- **v0.2 (this release)** voice control (Web Speech API), profile-aware gamepad mapper (drone / mobile / arm / quadruped / humanoid), native OS notifications, gamepad rumble on safety events, CI workflow, cross-platform release pipeline
- **v0.3** native whisper.cpp STT so macOS gets parity, mission macros recorder, camera feed pane (RTSP/WebRTC), embedded MuJoCo viewer in the robot detail page
- **v0.4** bidirectional MCP bridge so a chat client like Claude Desktop can drive the same robot the desktop app is showing
- **v1.0** code-signed builds for macOS / Windows, auto-update via Tauri updater plugin

## License

MIT. See [LICENSE](LICENSE).

## See also

- **[ghostloop](https://github.com/joemunene-by/ghostloop)**: the runtime + library (`pip install ghostloop`)
- **[ghostloop-ui](https://github.com/joemunene-by/ghostloop-ui)**: the Next.js frontend (also embeddable, no Tauri)
- **[Live demo](https://huggingface.co/spaces/Ghostgim/ghostloop-demo)**: Gradio control panel, no install
