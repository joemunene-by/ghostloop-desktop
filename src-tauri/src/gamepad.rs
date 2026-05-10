// Gamepad input + force-feedback (rumble).
//
// Two surfaces:
//   - `spawn_listener`: passive thread that polls all connected
//     controllers via gilrs at ~120 Hz and forwards button/axis
//     events to the WebView over the `gamepad` Tauri event channel.
//     The frontend converts the events into Intent dispatches.
//   - `rumble_pulse` Tauri command: called by the WebView whenever
//     it wants to signal a safety event (geofence block, force-cap
//     trip, HITL escalation, emergency stop). The WebView decides
//     intensity + duration; this side stays dumb on purpose so a
//     future UI knob doesn't require Rust changes.
//
// gilrs' force-feedback API is gated behind the `serde-serialize`
// feature only because that pulls in the rusty_ff_subsystem on
// non-mobile targets. We share a single Mutex'd Gilrs handle between
// the listener thread and the rumble command via `OnceLock`.

use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use gilrs::ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Ticks};
use gilrs::{Event, EventType, GamepadId, Gilrs};
use serde::{Deserialize, Serialize};
use tauri::Emitter;

#[derive(Debug, Serialize, Clone)]
struct GamepadEvent {
    pad_id: usize,
    pad_name: String,
    kind: &'static str,
    code: String,
    value: f32,
}

/// Shared Gilrs handle. We initialise it once in the listener thread
/// and reuse it from `rumble_pulse` so we don't fight over the audio
/// subsystem on macOS.
static GILRS: OnceLock<Mutex<Gilrs>> = OnceLock::new();

fn gilrs_handle() -> Result<&'static Mutex<Gilrs>, String> {
    if let Some(h) = GILRS.get() {
        return Ok(h);
    }
    let g = Gilrs::new().map_err(|e| format!("gilrs init failed: {}", e))?;
    let _ = GILRS.set(Mutex::new(g));
    Ok(GILRS.get().expect("just initialised"))
}

pub fn spawn_listener(app: tauri::AppHandle) {
    std::thread::spawn(move || {
        let mutex = match gilrs_handle() {
            Ok(g) => g,
            Err(e) => {
                eprintln!("[gamepad] {}", e);
                return;
            }
        };

        // Initial inventory log so the user knows what's connected.
        {
            let gilrs = match mutex.lock() {
                Ok(g) => g,
                Err(e) => {
                    eprintln!("[gamepad] poisoned lock at startup: {}", e);
                    return;
                }
            };
            for (id, gp) in gilrs.gamepads() {
                eprintln!("[gamepad] connected #{}: {}", usize::from(id), gp.name());
            }
        }

        loop {
            let mut events = Vec::new();
            {
                let mut gilrs = match mutex.lock() {
                    Ok(g) => g,
                    Err(_) => {
                        std::thread::sleep(Duration::from_millis(50));
                        continue;
                    }
                };
                while let Some(Event { id, event, .. }) = gilrs.next_event() {
                    events.push((id, event));
                }
            }
            for (id, event) in events {
                let pad_id = usize::from(id);
                let pad_name = pad_name(mutex, id);
                let payload = match event {
                    EventType::ButtonPressed(btn, _) => GamepadEvent {
                        pad_id,
                        pad_name,
                        kind: "button_press",
                        code: format!("{:?}", btn),
                        value: 1.0,
                    },
                    EventType::ButtonReleased(btn, _) => GamepadEvent {
                        pad_id,
                        pad_name,
                        kind: "button_release",
                        code: format!("{:?}", btn),
                        value: 0.0,
                    },
                    EventType::AxisChanged(axis, value, _) => GamepadEvent {
                        pad_id,
                        pad_name,
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

fn pad_name(mutex: &Mutex<Gilrs>, id: GamepadId) -> String {
    match mutex.lock() {
        Ok(g) => g.gamepad(id).name().to_string(),
        Err(_) => String::new(),
    }
}

#[derive(Debug, Deserialize)]
pub struct RumbleRequest {
    /// Magnitude in [0, 1]. Mapped onto the strong + weak motors equally.
    #[serde(default = "default_intensity")]
    intensity: f32,
    /// Pulse duration in milliseconds. Clamped to [50, 2000].
    #[serde(default = "default_duration_ms")]
    duration_ms: u32,
    /// Optional gamepad index. None = first connected pad.
    pad_id: Option<usize>,
}

fn default_intensity() -> f32 {
    0.6
}
fn default_duration_ms() -> u32 {
    250
}

#[tauri::command]
pub fn rumble_pulse(req: RumbleRequest) -> Result<(), String> {
    let mutex = gilrs_handle()?;
    let mut gilrs = mutex.lock().map_err(|e| format!("lock: {}", e))?;

    let intensity = req.intensity.clamp(0.0, 1.0);
    let duration_ms = req.duration_ms.clamp(50, 2000);

    let mut target_id: Option<GamepadId> = None;
    for (id, gp) in gilrs.gamepads() {
        if gp.is_connected() && (req.pad_id.is_none() || req.pad_id == Some(usize::from(id))) {
            target_id = Some(id);
            break;
        }
    }
    let Some(target_id) = target_id else {
        return Err("no gamepad connected".to_string());
    };

    let mag = (intensity * u16::MAX as f32) as u16;
    let strong = BaseEffect {
        kind: BaseEffectType::Strong { magnitude: mag },
        scheduling: Replay {
            play_for: Ticks::from_ms(duration_ms),
            ..Default::default()
        },
        envelope: Default::default(),
    };
    let weak = BaseEffect {
        kind: BaseEffectType::Weak { magnitude: mag },
        scheduling: Replay {
            play_for: Ticks::from_ms(duration_ms),
            ..Default::default()
        },
        envelope: Default::default(),
    };

    let effect = EffectBuilder::new()
        .add_effect(strong)
        .add_effect(weak)
        .gamepads(&[target_id])
        .finish(&mut gilrs)
        .map_err(|e| format!("effect build failed: {}", e))?;

    effect.play().map_err(|e| format!("play failed: {}", e))?;
    Ok(())
}
