//! Cross-platform clipboard change watcher.
//!
//! Polls clipboard text via `arboard` every 150ms on all platforms and emits
//! a "clipboard-changed" event when it changes to a non-empty value. The
//! popup is not triggered automatically — the user presses the selection
//! hotkey, at which point the frontend reads the latest clipboard.

use tauri::{AppHandle, Emitter};

const POLL_INTERVAL_MS: u64 = 150;

pub fn start(app: AppHandle) {
    std::thread::Builder::new()
        .name("wispet-clipboard-watcher".to_string())
        .spawn(move || {
            let mut last_text = String::new();

            loop {
                std::thread::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS));

                if let Some(text) = read_clipboard_text() {
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() && trimmed != last_text {
                        last_text = trimmed.clone();
                        let _ = app.emit("clipboard-changed", trimmed);
                    }
                }
            }
        })
        .expect("Failed to spawn clipboard watcher thread");
}

/// Read current clipboard text, returning None on error.
fn read_clipboard_text() -> Option<String> {
    let mut ctx = arboard::Clipboard::new().ok()?;
    ctx.get_text().ok()
}
