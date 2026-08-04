/**
 * ipc.js — Tauri IPC bridge
 * Wraps invoke() and event.listen() with typed, documented helpers.
 */

// Tauri v2 with withGlobalTauri: invoke lives at __TAURI__.core.invoke
// (not __TAURI__.invoke, which was the v1 layout); event stays top-level.
const { invoke } = window.__TAURI__.core;
const { event } = window.__TAURI__;

// ── Lookup ────────────────────────────────────────────────────────────────────

/**
 * @param {string} query
 * @returns {Promise<ProviderResult[]>}
 *
 * @typedef {Object} ProviderResult
 * @property {string} provider_id
 * @property {string} provider_label
 * @property {'dict'|'translation'|'article'} kind
 * @property {string} content
 * @property {string|null} phonetic
 * @property {string|null} source_lang
 */
export async function lookup(query) {
  return invoke('lookup', { query });
}

// ── Config ────────────────────────────────────────────────────────────────────

/** @returns {Promise<Config>} */
export async function getConfig() {
  return invoke('get_config');
}

/** @param {Config} config @returns {Promise<void>} */
export async function saveConfig(config) {
  return invoke('save_config', { newCfg: config });
}

/** @returns {Promise<string>} path to config.toml */
export async function getConfigPath() {
  return invoke('get_config_path');
}

// ── Clipboard ─────────────────────────────────────────────────────────────────

/** @returns {Promise<string>} */
export async function getClipboard() {
  return invoke('get_clipboard');
}

// ── Window ────────────────────────────────────────────────────────────────────

export async function showMainWindow() {
  return invoke('show_main_window');
}

export async function hideMainWindow() {
  return invoke('hide_main_window');
}

export async function hidePopup() {
  return invoke('hide_popup');
}

// ── Events ────────────────────────────────────────────────────────────────────

/**
 * Listen for clipboard changes (text changed to non-empty value).
 * The popup is NOT triggered automatically — frontend must wait for hotkey.
 * @param {(text: string) => void} handler
 * @returns {Promise<() => void>} unlisten function
 */
export function onClipboardChange(handler) {
  return event.listen('clipboard-changed', e => handler(e.payload));
}

/**
 * Listen for show-popup events (triggered by global hotkey in Rust).
 * @param {(payload: {query: string, x: number, y: number}) => void} handler
 */
export function onShowPopup(handler) {
  return event.listen('show-popup', e => handler(e.payload));
}

/**
 * Broadcast a query from the popup window to the main window. WebviewWindow#eval
 * is a Rust-only API (not exposed on the JS-side window object), so we use
 * the event system instead of cross-window scripting.
 * @param {string} query
 */
export function emitOpenQuery(query) {
  return event.emit('open-query', query);
}

/**
 * Listen for a query broadcast from the popup window (see emitOpenQuery).
 * @param {(query: string) => void} handler
 */
export function onOpenQuery(handler) {
  return event.listen('open-query', e => handler(e.payload));
}
