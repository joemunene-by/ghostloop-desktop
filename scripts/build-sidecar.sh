#!/usr/bin/env bash
#
# Build the ghostloop-server sidecar binary that ships with the desktop
# bundle. PyInstaller -> single-file executable, named per Tauri's
# expected platform suffix (e.g. ghostloop-server-x86_64-apple-darwin).
#
# Run this BEFORE `npm run build` so the binary lands in
# src-tauri/binaries/ for inclusion in the bundle.

set -euo pipefail

cd "$(dirname "$0")/.."

# Resolve Tauri's target triple — it expects sidecars named
# `<sidecar>-<rustc-target-triple>` so they're picked per platform.
TARGET_TRIPLE=$(rustc -Vv | sed -n 's/host: //p')

echo "[sidecar] target: $TARGET_TRIPLE"

# Install build deps in a clean venv to avoid polluting the dev env.
python3 -m venv .build-venv
. .build-venv/bin/activate
pip install --upgrade pip
pip install pyinstaller
pip install ghostloop fastapi uvicorn

mkdir -p src-tauri/binaries

# Wrap ghostloop's MCP server in a small launcher PyInstaller can package.
cat > .build-venv/sidecar_launcher.py <<'PY'
"""Sidecar entry point packaged by PyInstaller.

Defaults match the Tauri tauri.conf.json beforeBuildCommand: serves the
production dashboard on 127.0.0.1:8000 with a Mock backend so the
desktop app can talk to it on first launch. The user replaces the
backend with their own server.py once they're past the demo stage.
"""

import os
import sys

if __name__ == "__main__":
    import uvicorn
    from ghostloop import GhostloopStore
    from ghostloop.dashboard import create_production_app

    db_path = os.environ.get(
        "GHOSTLOOP_DB",
        os.path.join(os.path.expanduser("~"), ".ghostloop", "store.db"),
    )
    os.makedirs(os.path.dirname(db_path), exist_ok=True)
    store = GhostloopStore(db_path)
    app, alarms = create_production_app(store=store, fleet=None)

    uvicorn.run(
        app,
        host=os.environ.get("GHOSTLOOP_HOST", "127.0.0.1"),
        port=int(os.environ.get("GHOSTLOOP_PORT", "8000")),
        log_level="info",
    )
PY

pyinstaller \
    --noconfirm \
    --onefile \
    --name "ghostloop-server-${TARGET_TRIPLE}" \
    --distpath src-tauri/binaries \
    .build-venv/sidecar_launcher.py

echo "[sidecar] -> src-tauri/binaries/ghostloop-server-${TARGET_TRIPLE}"
deactivate
