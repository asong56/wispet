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

export async function hideMainWindow() {
  return invoke('hide_main_window');
}

// Rust emits this once per hotkey press — see register_shortcuts() in
// main.rs. The payload is always an empty string; there is no pre-fill
// path (no clipboard/selection lookup). Kept as a payload for forward
// compatibility rather than a bare no-arg event.
export function onQuery(handler) {
  return event.listen('wispet-query', e => handler(e.payload));
}
