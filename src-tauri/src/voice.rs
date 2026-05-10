// Voice control plumbing.
//
// v0.2 uses the Web Speech API inside the WebView for STT. That gets
// us a working pipeline on WebView2 (Windows) and webkit2gtk (Linux),
// and gracefully degrades to "voice unavailable" on macOS WKWebView.
//
// The Rust side here exposes one query (`voice_state`) so the
// frontend can decide whether to show a "voice on" toggle or an
// "install macOS dictation" fallback. v0.3 replaces this with a
// native engine (whisper-rs or vosk-rs) so the experience matches
// across platforms; the Tauri-side interface stays the same.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct VoiceState {
    /// Best-effort guess at whether the embedded WebView supports
    /// SpeechRecognition. The frontend still feature-detects at
    /// runtime; this just lets the UI render the right message
    /// before that probe completes.
    web_speech_likely: bool,
    /// Pretty-printed platform label for the settings page.
    platform: &'static str,
    /// Whether a native STT engine is bundled. False today; flips to
    /// true in v0.3 once whisper-rs lands.
    native_engine: bool,
    /// Suggested wake phrase. Centralized here so the README, the
    /// UI banner, and the listener stay in sync.
    wake_phrase: &'static str,
}

#[tauri::command]
pub fn voice_state() -> VoiceState {
    VoiceState {
        web_speech_likely: web_speech_likely_supported(),
        platform: platform_label(),
        native_engine: false,
        wake_phrase: "ghostloop",
    }
}

const fn web_speech_likely_supported() -> bool {
    // Tauri's WebView per OS:
    //   macOS:   WKWebView, no SpeechRecognition
    //   Windows: WebView2 (Chromium), has webkitSpeechRecognition
    //   Linux:   webkit2gtk, has SpeechRecognition behind a flag
    //
    // This is just a hint; the frontend feature-detects for real.
    cfg!(any(target_os = "windows", target_os = "linux"))
}

const fn platform_label() -> &'static str {
    if cfg!(target_os = "macos") {
        "macOS"
    } else if cfg!(target_os = "windows") {
        "Windows"
    } else if cfg!(target_os = "linux") {
        "Linux"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_state_returns_known_platform() {
        let s = voice_state();
        assert!(
            matches!(s.platform, "macOS" | "Windows" | "Linux" | "unknown"),
            "unexpected platform label: {}",
            s.platform
        );
        assert_eq!(s.wake_phrase, "ghostloop");
        assert!(!s.native_engine);
    }
}
