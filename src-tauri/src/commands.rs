use crate::{config, provider, AppState};
use tauri::{AppHandle, Manager, State};

#[tauri::command]
pub async fn lookup(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<provider::ProviderResult>, String> {
    if query.trim().is_empty() {
        return Ok(vec![]);
    }

    let providers = state.providers.lock().await;
    let results = provider::dispatch(&providers, &query).await;
    Ok(results)
}

#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<config::Config, String> {
    let cfg = state.config.lock().await;
    Ok(cfg.clone())
}

#[tauri::command]
pub async fn save_config(
    new_cfg: config::Config,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    config::save(&new_cfg).map_err(|e| e.to_string())?;

    let new_providers = crate::build_providers(&new_cfg);
    {
        let mut providers = state.providers.lock().await;
        *providers = new_providers;
    }

    // Update state before re-registering shortcuts so anything reading
    // state during registration sees the new values.
    *state.config.lock().await = new_cfg.clone();

    crate::register_shortcuts(&app, &new_cfg)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_clipboard() -> String {
    use arboard::Clipboard;
    Clipboard::new()
        .and_then(|mut c| c.get_text())
        .unwrap_or_default()
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }
}

#[tauri::command]
pub fn show_popup(app: AppHandle, x: i32, y: i32, query: String) {
    use tauri::PhysicalPosition;

    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.eval(&format!(
            "window.__wispet_query__ = {}; if (window.onWispetQuery) onWispetQuery(window.__wispet_query__);",
            serde_json::to_string(&query).unwrap_or_default()
        ));
        let _ = win.set_position(PhysicalPosition::new(x, y));
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[tauri::command]
pub fn hide_popup(app: AppHandle) {
    if let Some(win) = app.get_webview_window("popup") {
        let _ = win.hide();
    }
}

#[tauri::command]
pub fn get_config_path() -> String {
    config::config_path().to_string_lossy().to_string()
}
