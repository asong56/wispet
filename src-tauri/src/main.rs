#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod commands;
mod config;
mod provider;
mod tray;

use anyhow::Result;
use provider::Provider;
use std::sync::Arc;
use tauri::{
    AppHandle, Emitter, Manager,
    webview::WebviewWindowBuilder,
};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tokio::sync::Mutex;

// ── App state ─────────────────────────────────────────────────────────────────

pub struct AppState {
    pub config: Mutex<config::Config>,
    pub providers: Mutex<Vec<Box<dyn Provider>>>,
}

// ── Provider builder ──────────────────────────────────────────────────────────

pub fn build_providers(cfg: &config::Config) -> Vec<Box<dyn Provider>> {
    let mut list = cfg.providers.list.clone();
    list.sort_by_key(|p| p.priority);

    let mut providers: Vec<Box<dyn Provider>> = Vec::new();

    for (i, entry) in list.iter().enumerate() {
        if !entry.enabled {
            continue;
        }

        let id = format!("{}_{}", entry.kind, i);
        let label = entry.label.clone().unwrap_or_else(|| entry.kind.clone());

        match entry.kind.as_str() {
            "mdx" => {
                if let Some(path) = &entry.path {
                    match provider::mdx::MdxProvider::new(path, label.clone(), id) {
                        Ok(p) => providers.push(Box::new(p)),
                        Err(e) => log::error!("Failed to load MDX dict at {}: {:#}", path, e),
                    }
                }
            }
            "deepl" => {
                if let Some(key) = &entry.api_key {
                    providers.push(Box::new(provider::deepl::DeeplProvider::new(
                        key.clone(),
                        entry.source_lang.clone().unwrap_or_else(|| "auto".into()),
                        entry.target_lang.clone().unwrap_or_else(|| "EN".into()),
                    )));
                } else {
                    log::warn!("DeepL provider missing api_key — skipping");
                }
            }
            "google" => {
                providers.push(Box::new(provider::google::GoogleProvider::new(
                    entry.source_lang.clone().unwrap_or_else(|| "auto".into()),
                    entry.target_lang.clone().unwrap_or_else(|| "zh-CN".into()),
                )));
            }
            "wikipedia" => {
                providers.push(Box::new(provider::wikipedia::WikipediaProvider::new(
                    entry.lang.clone().unwrap_or_else(|| "en".into()),
                )));
            }
            other => {
                log::warn!("Unknown provider type '{}' — skipping", other);
            }
        }
    }

    providers
}

// ── Shortcut registration ─────────────────────────────────────────────────────

pub fn register_shortcuts(app: &AppHandle, cfg: &config::Config) -> Result<()> {
    let mgr = app.global_shortcut();

    // Unregister all before re-registering
    mgr.unregister_all()?;

    let hotkey_main = cfg.general.hotkey_main.clone();
    let hotkey_sel = cfg.general.hotkey.clone();

    let app_main = app.clone();
    mgr.on_shortcut(
        parse_shortcut(&hotkey_main)?,
        move |_app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                toggle_main_window(&app_main);
            }
        },
    )?;

    let app_sel = app.clone();
    mgr.on_shortcut(
        parse_shortcut(&hotkey_sel)?,
        move |_app, _shortcut, event| {
            if event.state() == ShortcutState::Pressed {
                trigger_popup(&app_sel);
            }
        },
    )?;

    Ok(())
}

fn toggle_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        if win.is_visible().unwrap_or(false) {
            let _ = win.hide();
        } else {
            let _ = win.show();
            let _ = win.set_focus();
        }
    }
}

fn trigger_popup(app: &AppHandle) {
    // Read clipboard
    let text = {
        use arboard::Clipboard;
        Clipboard::new()
            .and_then(|mut c| c.get_text())
            .unwrap_or_default()
    };

    if text.trim().is_empty() {
        return;
    }

    // Get mouse position for popup placement
    let pos = get_cursor_position();

    let _ = app.emit("show-popup", serde_json::json!({
        "query": text.trim(),
        "x": pos.0,
        "y": pos.1,
    }));
}

fn get_cursor_position() -> (i32, i32) {
    // Platform-specific cursor position
    // Fallback to (0, 0) if unavailable
    #[cfg(target_os = "macos")]
    {
        // Use NSEvent.mouseLocation via objc (simplified — returns (0,0) for now)
        (0, 0)
    }
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;
        use windows::Win32::Foundation::POINT;
        unsafe {
            let mut pt = POINT::default();
            if GetCursorPos(&mut pt).is_ok() {
                return (pt.x, pt.y);
            }
        }
        (0, 0)
    }
    #[cfg(target_os = "linux")]
    { (0, 0) }
}

/// Parse a shortcut string like "Alt+D" or "Ctrl+Shift+W"
fn parse_shortcut(s: &str) -> Result<Shortcut> {
    let parts: Vec<&str> = s.split('+').collect();
    let mut mods = Modifiers::empty();
    let mut key_code = None;

    for part in &parts {
        match part.to_lowercase().as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" | "option" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "super" | "meta" | "cmd" | "command" => mods |= Modifiers::SUPER,
            k => {
                key_code = Some(str_to_code(k)?);
            }
        }
    }

    let code = key_code.ok_or_else(|| anyhow::anyhow!("Shortcut '{}' has no key", s))?;
    Ok(Shortcut::new(Some(mods), code))
}

fn str_to_code(s: &str) -> Result<Code> {
    let code = match s {
        "a" => Code::KeyA, "b" => Code::KeyB, "c" => Code::KeyC, "d" => Code::KeyD,
        "e" => Code::KeyE, "f" => Code::KeyF, "g" => Code::KeyG, "h" => Code::KeyH,
        "i" => Code::KeyI, "j" => Code::KeyJ, "k" => Code::KeyK, "l" => Code::KeyL,
        "m" => Code::KeyM, "n" => Code::KeyN, "o" => Code::KeyO, "p" => Code::KeyP,
        "q" => Code::KeyQ, "r" => Code::KeyR, "s" => Code::KeyS, "t" => Code::KeyT,
        "u" => Code::KeyU, "v" => Code::KeyV, "w" => Code::KeyW, "x" => Code::KeyX,
        "y" => Code::KeyY, "z" => Code::KeyZ,
        "0" => Code::Digit0, "1" => Code::Digit1, "2" => Code::Digit2,
        "3" => Code::Digit3, "4" => Code::Digit4, "5" => Code::Digit5,
        "6" => Code::Digit6, "7" => Code::Digit7, "8" => Code::Digit8, "9" => Code::Digit9,
        "f1" => Code::F1, "f2" => Code::F2, "f3" => Code::F3, "f4" => Code::F4,
        "f5" => Code::F5, "f6" => Code::F6, "f7" => Code::F7, "f8" => Code::F8,
        "f9" => Code::F9, "f10" => Code::F10, "f11" => Code::F11, "f12" => Code::F12,
        "space" => Code::Space, "enter" | "return" => Code::Enter,
        "escape" | "esc" => Code::Escape,
        "tab" => Code::Tab,
        "backspace" => Code::Backspace,
        "delete" => Code::Delete,
        _ => anyhow::bail!("Unknown key code: '{}'", s),
    };
    Ok(code)
}

// ── Main ──────────────────────────────────────────────────────────────────────

fn main() {
    env_logger::init();

    let cfg = config::load_or_create().unwrap_or_else(|e| {
        log::error!("Config error: {:#} — using defaults", e);
        config::Config::default()
    });

    let providers = build_providers(&cfg);

    let state = AppState {
        config: Mutex::new(cfg.clone()),
        providers: Mutex::new(providers),
    };

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let handle = app.handle().clone();

            // System tray
            tray::setup(&handle)?;

            // Popup window (hidden, created once, reused)
            WebviewWindowBuilder::new(app, "popup", tauri::WebviewUrl::App("popup.html".into()))
                .title("Wispet")
                .inner_size(420.0, 480.0)
                .decorations(false)
                .transparent(true)
                .shadow(true)
                .always_on_top(true)
                .skip_taskbar(true)
                .visible(false)
                .build()?;

            // Global shortcuts
            let cfg_snap = app.state::<AppState>().config.blocking_lock().clone();
            register_shortcuts(&handle, &cfg_snap)
                .unwrap_or_else(|e| log::error!("Shortcut registration failed: {:#}", e));

            // Clipboard watcher
            clipboard::start(handle.clone());

            // Forward show-popup event to the popup window
            let fwd = handle.clone();
            handle.listen("show-popup", move |event| {
                if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                    let query = payload["query"].as_str().unwrap_or("").to_string();
                    let x = payload["x"].as_i64().unwrap_or(0) as i32;
                    let y = payload["y"].as_i64().unwrap_or(0) as i32;

                    if let Some(win) = fwd.get_webview_window("popup") {
                        let _ = win.eval(&format!(
                            "window.__wispet_query__ = {}; if (window.onWispetQuery) onWispetQuery(window.__wispet_query__);",
                            serde_json::to_string(&query).unwrap_or_default()
                        ));
                        let _ = win.set_position(tauri::PhysicalPosition::new(x + 16, y + 24));
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::lookup,
            commands::get_config,
            commands::save_config,
            commands::get_clipboard,
            commands::show_main_window,
            commands::hide_main_window,
            commands::show_popup,
            commands::hide_popup,
            commands::get_config_path,
        ])
        .on_window_event(|window, event| {
            // Hide main window on close instead of quitting
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" || window.label() == "popup" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("Error running Wispet");
}
