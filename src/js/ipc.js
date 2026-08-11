// Tauri v2 with withGlobalTauri: invoke lives at __TAURI__.core.invoke
// (not __TAURI__.invoke, the v1 layout); event stays top-level.
const { invoke } = window.__TAURI__.core;
const { event } = window.__TAURI__;

export async function lookup(query) {
  return invoke('lookup', { query });
}

export async function getConfig() {
  return invoke('get_config');
}

export async function saveConfig(config) {
  return invoke('save_config', { newCfg: config });
}

export async function getConfigPath() {
  return invoke('get_config_path');
}

export async function getClipboard() {
  return invoke('get_clipboard');
}

export async function showMainWindow() {
  return invoke('show_main_window');
}

export async function hideMainWindow() {
  return invoke('hide_main_window');
}

export async function hidePopup() {
  return invoke('hide_popup');
}

export function onClipboardChange(handler) {
  return event.listen('clipboard-changed', e => handler(e.payload));
}

export function onShowPopup(handler) {
  return event.listen('show-popup', e => handler(e.payload));
}

// WebviewWindow#eval is Rust-only, so cross-window messaging uses events.
export function emitOpenQuery(query) {
  return event.emit('open-query', query);
}

export function onOpenQuery(handler) {
  return event.listen('open-query', e => handler(e.payload));
}
