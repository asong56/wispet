/**
 * main.js — Wispet main window
 *
 * View routing:  #results-view  ↔  #settings-view
 * Search:        debounced input → lookup() → renderResults()
 * Keyboard:      Esc (clear/hide), Cmd+, (settings)
 */

import { lookup, getConfig, saveConfig, getConfigPath, onClipboardChange } from './ipc.js';
import { renderResult, renderLoading } from './render.js';
import { initSettings } from './settings.js';

// ── DOM refs ──────────────────────────────────────────────────────────────────

const searchInput   = document.getElementById('search-input');
const resultsView   = document.getElementById('results-view');
const settingsView  = document.getElementById('settings-view');
const emptyState    = document.getElementById('empty-state');
const btnSettings   = document.getElementById('btn-settings');
const btnClose      = document.getElementById('btn-close');
const clearBtn      = document.getElementById('btn-clear');

// ── State ─────────────────────────────────────────────────────────────────────

let currentQuery   = '';
let debounceTimer  = null;
let activeView     = 'results'; // 'results' | 'settings'

// ── Init ──────────────────────────────────────────────────────────────────────

async function init() {
  const config = await getConfig();
  applyTheme(config.general.theme);

  await initSettings(config, onConfigSaved);

  // Focus search immediately
  searchInput.focus();

  // Listen for clipboard changes — just track latest; no auto-popup
  onClipboardChange(_text => {
    // Could prefill on future keypress — for now just a no-op hook
  });

  // Wire events
  searchInput.addEventListener('input', onSearchInput);
  searchInput.addEventListener('keydown', onSearchKeydown);
  btnSettings.addEventListener('click', toggleSettings);
  btnClose.addEventListener('click', () => window.__TAURI__.window.getCurrent().hide());
  clearBtn?.addEventListener('click', clearSearch);

  // Tray/shortcut can signal navigation
  window.onWispetNav = (view) => {
    if (view === 'settings') showSettings();
  };

  // Global keyboard
  document.addEventListener('keydown', onGlobalKeydown);
}

// ── Search ────────────────────────────────────────────────────────────────────

function onSearchInput() {
  const q = searchInput.value.trim();
  clearBtn && (clearBtn.style.display = q ? 'flex' : 'none');

  clearTimeout(debounceTimer);
  if (!q) {
    currentQuery = '';
    showEmpty();
    return;
  }
  debounceTimer = setTimeout(() => doLookup(q), 280);
}

function onSearchKeydown(e) {
  if (e.key === 'Escape') {
    if (searchInput.value) {
      clearSearch();
    } else {
      window.__TAURI__.window.getCurrent().hide();
    }
  }
  // Cmd/Ctrl+, → settings
  if ((e.metaKey || e.ctrlKey) && e.key === ',') {
    e.preventDefault();
    toggleSettings();
  }
}

function onGlobalKeydown(e) {
  // Any printable character not in a form field → focus search
  if (
    activeView === 'results' &&
    e.key.length === 1 &&
    !e.metaKey && !e.ctrlKey && !e.altKey &&
    document.activeElement !== searchInput
  ) {
    searchInput.focus();
  }
}

async function doLookup(query) {
  if (query === currentQuery) return;
  currentQuery = query;

  showResultsView();
  renderLoadingState();

  try {
    const results = await lookup(query);
    if (query !== currentQuery) return; // stale
    renderResults(results, query);
  } catch (err) {
    renderError(err);
  }
}

function clearSearch() {
  searchInput.value = '';
  currentQuery = '';
  clearBtn && (clearBtn.style.display = 'none');
  showEmpty();
  searchInput.focus();
}

// ── Rendering ─────────────────────────────────────────────────────────────────

function showEmpty() {
  resultsView.innerHTML = '';
  emptyState.style.display = 'flex';
}

function showResultsView() {
  emptyState.style.display = 'none';
  activeView = 'results';
  resultsView.style.display = 'block';
  settingsView.style.display = 'none';
}

function renderLoadingState() {
  resultsView.innerHTML = '';
  // We don't know which providers are enabled here; show a generic spinner
  const skeleton = renderLoading('Looking up…');
  resultsView.appendChild(skeleton);
}

function renderResults(results, query) {
  resultsView.innerHTML = '';

  if (!results || results.length === 0) {
    const noResult = document.createElement('section');
    noResult.className = 'result';
    noResult.innerHTML = `
      <div class="provider-header">
        <span class="provider-name">No results</span>
      </div>
      <div class="dict-content" style="color:var(--color-text-muted);font-size:var(--text-sm)">
        No provider returned a result for <em>${escapeHtml(query)}</em>.
        Check your provider settings or try a different query.
      </div>`;
    resultsView.appendChild(noResult);
    return;
  }

  for (const result of results) {
    resultsView.appendChild(renderResult(result));
  }
}

function renderError(err) {
  resultsView.innerHTML = '';
  const sec = document.createElement('section');
  sec.className = 'result';
  sec.innerHTML = `
    <div class="provider-header">
      <span class="provider-name" style="color:var(--color-text-muted)">Error</span>
    </div>
    <div class="dict-content" style="color:var(--color-text-muted);font-size:var(--text-sm)">
      ${escapeHtml(String(err?.message ?? err))}
    </div>`;
  resultsView.appendChild(sec);
}

// ── View routing ──────────────────────────────────────────────────────────────

function toggleSettings() {
  if (activeView === 'settings') {
    showResultsView();
    searchInput.focus();
    btnSettings.setAttribute('aria-pressed', 'false');
  } else {
    showSettings();
  }
}

function showSettings() {
  activeView = 'settings';
  emptyState.style.display = 'none';
  resultsView.style.display = 'none';
  settingsView.style.display = 'flex';
  btnSettings.setAttribute('aria-pressed', 'true');
}

async function onConfigSaved(newConfig) {
  applyTheme(newConfig.general.theme);
  // Return to search after save
  showResultsView();
  searchInput.focus();
}

// ── Theme ─────────────────────────────────────────────────────────────────────

function applyTheme(theme) {
  if (theme === 'auto') {
    document.body.removeAttribute('data-theme');
  } else {
    document.body.setAttribute('data-theme', theme);
  }
}

// ── Util ──────────────────────────────────────────────────────────────────────

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

// ── Boot ──────────────────────────────────────────────────────────────────────

init().catch(err => console.error('[Wispet] init error:', err));
