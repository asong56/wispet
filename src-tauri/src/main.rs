#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod clipboard;
mod commands;
mod config;
mod provider;
mod tray;

use anyhow::Result;
use provider::Provider;
use tauri::{
    AppHandle, Emitter, Listener, Manager,
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
    let text = {
        use arboard::Clipboard;
        Clipboard::new()
            .and_then(|mut c| c.get_text())
            .unwrap_or_default()
    };

    if text.trim().is_empty() {
        return;
    }

    let pos = get_cursor_position();

    let _ = app.emit("show-popup", serde_json::json!({
        "query": text.trim(),
        "x": pos.0,
        "y": pos.1,
    }));
}

fn get_cursor_position() -> (i32, i32) {
    // Linux: queries X11, so this returns (0, 0) under a pure-Wayland session.
    use mouse_position::mouse_position::Mouse;
    match Mouse::get_mouse_position() {
        Mouse::Position { x, y } => (x, y),
        Mouse::Error => (0, 0),
    }
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

// ── wispet:// media protocol ──────────────────────────────────────────────────
// Resolves wispet://mdd/<category>/<name> requests (rewritten from sound://
// links by sanitize_mdx_html in provider/mdx.rs) against the loaded MDX
// dictionaries' sibling .mdd files.

/// Matches loosely on the "mdd/" marker rather than a strict scheme/host/path
/// split, since custom-protocol URL shape varies slightly across platforms.
fn extract_wispet_resource_name(uri: &str) -> Option<String> {
    let after_mdd = uri.split("mdd/").nth(1)?;
    let raw_name = after_mdd.split('/').nth(1).unwrap_or(after_mdd);
    if raw_name.is_empty() {
        return None;
    }
    Some(
        urlencoding::decode(raw_name)
            .map(|c| c.into_owned())
            .unwrap_or_else(|_| raw_name.to_string()),
    )
}

/// Re-parses each candidate .mdd's key index per call rather than caching —
/// runs off the main thread, so simple and correct beats fast here unless it
/// becomes a bottleneck for very large dictionaries.
fn resolve_mdd_resource(app: &AppHandle, name: &str) -> Option<Vec<u8>> {
    let state = app.state::<AppState>();
    let cfg = state.config.blocking_lock().clone();

    for entry in &cfg.providers.list {
        if entry.kind != "mdx" {
            continue;
        }
        let Some(mdx_path) = &entry.path else { continue };
        let mdx_path = std::path::Path::new(mdx_path);

        for ext in ["mdd", "MDD"] {
            let mdd_path = mdx_path.with_extension(ext);
            if !mdd_path.is_file() {
                continue;
            }
            match provider::mdx::MddDict::open(&mdd_path) {
                Ok(mdd) => match mdd.lookup_resource(name) {
                    Ok(Some(bytes)) => return Some(bytes),
                    Ok(None) => {}
                    Err(e) => log::warn!("MDD lookup error in {}: {:#}", mdd_path.display(), e),
                },
                Err(e) => log::warn!("Failed to open MDD file {}: {:#}", mdd_path.display(), e),
            }
        }
    }
    None
}

fn guess_mime(name: &str) -> &'static str {
    match name.rsplit('.').next().unwrap_or("").to_lowercase().as_str() {
        "mp3" => "audio/mpeg",
        "wav" => "audio/wav",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "css" => "text/css",
        "js" => "application/javascript",
        _ => "application/octet-stream",
    }
}

fn wispet_not_found() -> http::Response<Vec<u8>> {
    http::Response::builder()
        .status(http::StatusCode::NOT_FOUND)
        .body(Vec::new())
        .unwrap()
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
        .register_asynchronous_uri_scheme_protocol("wispet", |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            let uri_str = request.uri().to_string();
            std::thread::spawn(move || {
                let response = match extract_wispet_resource_name(&uri_str) {
                    Some(name) => match resolve_mdd_resource(&app, &name) {
                        Some(bytes) => http::Response::builder()
                            .header(http::header::CONTENT_TYPE, guess_mime(&name))
                            .body(bytes)
                            .unwrap(),
                        None => wispet_not_found(),
                    },
                    None => wispet_not_found(),
                };
                responder.respond(response);
            });
        })
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
