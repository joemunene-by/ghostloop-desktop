# ghostloop-desktop

Native desktop control panel for [ghostloop](https://github.com/joemunene-by/ghostloop). Tauri 2 shell wrapping the [ghostloop-ui](https://github.com/joemunene-by/ghostloop-ui) Next.js frontend, with native gamepad input and a sidecar Python runtime that serves the dashboard backend on first launch.

## What you get

- **Single-file desktop app** for macOS / Windows / Linux. No `pip install`, no `npm run dev` — double-click to run.
- **Sidecar Python runtime** packaged via PyInstaller. Serves the ghostloop production dashboard on `127.0.0.1:8000` with a SQLite store at `~/.ghostloop/store.db`.
- **Native gamepad input** via [gilrs](https://gitlab.com/gilrs-project/gilrs). Forwards normalized axis / button events to the embedded WebView over the `gamepad` Tauri event channel — so a virtual joystick maps to a real Xbox / PS controller.
- **System-tray integration** so the app stays accessible across desktops.
- **Hot-key for emergency stop** wired to `intervention_emergency_stop`, which the frontend converts into a POST against the dashboard's intervention endpoint.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│  Tauri (Rust shell)                                     │
│  ├── WebView (Next.js — ghostloop-ui)                   │
│  ├── Sidecar process (Python — ghostloop production    │
│  │   dashboard via PyInstaller'd binary)                │
│  └── Native event bridge:                               │
│      ├── gamepad → emit("gamepad", {pad, kind, code, value})  │
│      ├── tray → emit("tray:click", item_id)             │
│      └── hotkey → emit("intervention:emergency_stop")   │
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
├── src-tauri/                 Rust shell + Tauri config
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── capabilities/main.json     fine-grained permissions
│   └── src/
│       ├── main.rs                process entry
│       └── lib.rs                 sidecar spawner + gamepad bridge
├── scripts/
│   └── build-sidecar.sh           PyInstaller -> ghostloop-server-<triple>
├── package.json
└── README.md
```

The frontend (Next.js) is consumed from a sibling clone of [ghostloop-ui](https://github.com/joemunene-by/ghostloop-ui), referenced via `tauri.conf.json`'s `beforeDevCommand` and `frontendDist`.

## Build (one-time setup)

Prerequisites: Rust (stable), Node 20+, Python 3.10+.

```bash
git clone https://github.com/joemunene-by/ghostloop-desktop
git clone https://github.com/joemunene-by/ghostloop-ui ../ghostloop-ui

cd ghostloop-desktop
npm install
cd ../ghostloop-ui && npm install && cd ../ghostloop-desktop

# Build the sidecar binary (one-time per machine).
./scripts/build-sidecar.sh
# -> src-tauri/binaries/ghostloop-server-<rustc-triple>

# Dev loop — opens the desktop window with hot-reloading frontend.
npm run dev

# Production bundle (.dmg / .AppImage / .deb / .msi / .nsis):
npm run build
```

The bundle artefacts land in `src-tauri/target/release/bundle/`.

## Native input details

`gilrs` polls every connected gamepad at ~120 Hz. Each `Event` is normalised into:

```ts
{ pad_id: number, pad_name: string, kind: "button_press" | "button_release" | "axis", code: string, value: number }
```

The frontend listens via `@tauri-apps/api/event`:

```ts
import { listen } from "@tauri-apps/api/event"

await listen<GamepadEvent>("gamepad", (e) => {
  if (e.payload.kind === "axis" && e.payload.code === "LeftStickX") {
    // map left-stick X to drive(angular_z) etc.
  }
})
```

## Roadmap

- **v0.1 (this release)** — shell + sidecar + gamepad + tray + e-stop hotkey
- **v0.2** — embedded MuJoCo viewer (WebGL) in the robot detail page
- **v0.3** — voice control via system speech-to-text (macOS / Windows native APIs, Whisper on Linux)
- **v0.4** — bidirectional MCP bridge so a chat client like Claude Desktop can drive the same robot the desktop app is showing
- **v1.0** — code-signed builds for macOS / Windows, auto-update via Tauri updater plugin

## License

MIT — see [LICENSE](LICENSE).

## See also

- **[ghostloop](https://github.com/joemunene-by/ghostloop)** — the runtime + library (`pip install ghostloop`)
- **[ghostloop-ui](https://github.com/joemunene-by/ghostloop-ui)** — the Next.js frontend (also embeddable, no Tauri)
- **[Live demo](https://huggingface.co/spaces/Ghostgim/ghostloop-demo)** — Gradio control panel, no install
