//! Polls clipboard text every 150ms and emits "clipboard-changed" when it
//! changes. The popup is not shown automatically — the frontend just keeps
//! the latest value until the user presses the selection hotkey.

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

fn read_clipboard_text() -> Option<String> {
    let mut ctx = arboard::Clipboard::new().ok()?;
    ctx.get_text().ok()
}
