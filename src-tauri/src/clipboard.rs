//! Cross-platform clipboard change watcher.
//!
//! Strategy:
//!   - macOS: poll `NSPasteboard.changeCount` every 150ms
//!   - Windows: poll via `GetClipboardSequenceNumber` every 150ms
//!   - Linux/X11: poll `arboard` clipboard every 150ms
//!   - Linux/Wayland: same poll (arboard handles both via x11/wayland backends)
//!
//! We emit a Tauri event "clipboard-changed" with the new text payload
//! whenever the clipboard text changes to a non-empty value.
//! The popup is NOT triggered automatically — the user must press the
//! selection hotkey, at which point the frontend reads the latest clipboard.

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
    // arboard works on all three platforms
    let mut ctx = arboard::Clipboard::new().ok()?;
    ctx.get_text().ok()
}
