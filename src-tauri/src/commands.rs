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

/// Returns the live config, loaded once at startup from wispet.toml.
/// There is no corresponding save_config command — configuration is edited
/// by hand in wispet.toml and picked up on next launch. This IPC call
/// exists only so the frontend can read display-only values (currently
/// none are consumed, kept for forward compatibility).
#[tauri::command]
pub async fn get_config(
    state: State<'_, AppState>,
) -> Result<config::Config, String> {
    let cfg = state.config.lock().await;
    Ok(cfg.clone())
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.hide();
    }
}
