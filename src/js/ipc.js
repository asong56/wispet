/**
 * ipc.js — Tauri IPC bridge
 * Wraps invoke() and event.listen() with typed, documented helpers.
 */

const { invoke, event } = window.__TAURI__;

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
