import { lookup, getConfig, saveConfig, getConfigPath, onClipboardChange, onOpenQuery } from './ipc.js';
import { renderResult, renderLoading } from './render.js';
import { initSettings } from './settings.js';

// Tauri v2 renamed window.getCurrent() -> getCurrentWindow()
const { getCurrentWindow } = window.__TAURI__.window;

const searchInput   = document.getElementById('search-input');
const resultsView   = document.getElementById('results-view');
const settingsView  = document.getElementById('settings-view');
const emptyState    = document.getElementById('empty-state');
const btnSettings   = document.getElementById('btn-settings');
const btnClose      = document.getElementById('btn-close');
const clearBtn      = document.getElementById('btn-clear');

let currentQuery   = '';
let debounceTimer  = null;
let activeView     = 'results'; // 'results' | 'settings'

async function init() {
  const config = await getConfig();
  applyTheme(config.general.theme);

  await initSettings(config, onConfigSaved);
  searchInput.focus();

  // Just track latest clipboard text; the popup is triggered by hotkey, not auto-shown
  onClipboardChange(_text => {});

  // Popup's "Open in Wispet" button broadcasts the query via this event
  onOpenQuery(query => {
    showResultsView();
    searchInput.value = query;
    searchInput.focus();
    clearBtn && (clearBtn.style.display = query ? 'flex' : 'none');
    doLookup(query.trim());
  });

  searchInput.addEventListener('input', onSearchInput);
  searchInput.addEventListener('keydown', onSearchKeydown);
  btnSettings.addEventListener('click', toggleSettings);
  btnClose.addEventListener('click', () => getCurrentWindow().hide());
  clearBtn?.addEventListener('click', clearSearch);

  // Set by tray.rs / commands.rs via win.eval() to trigger navigation
  window.onWispetNav = (view) => {
    if (view === 'settings') showSettings();
  };

  document.addEventListener('keydown', onGlobalKeydown);
}

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
      getCurrentWindow().hide();
    }
  }
  if ((e.metaKey || e.ctrlKey) && e.key === ',') {
    e.preventDefault();
    toggleSettings();
  }
}

function onGlobalKeydown(e) {
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
  showResultsView();
  searchInput.focus();
}

function applyTheme(theme) {
  if (theme === 'auto') {
    document.body.removeAttribute('data-theme');
  } else {
    document.body.setAttribute('data-theme', theme);
  }
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

init().catch(err => console.error('[Wispet] init error:', err));
