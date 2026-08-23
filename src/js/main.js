import { lookup, getConfig, onQuery, hideMainWindow } from './ipc.js';
import { renderResult, renderLoading } from './render.js';

// Tauri v2 renamed window.getCurrent() -> getCurrentWindow()
const { getCurrentWindow, LogicalSize, LogicalPosition } = window.__TAURI__.window;

const searchInput = document.getElementById('search-input');
const resultsView = document.getElementById('results-view');
const emptyState  = document.getElementById('empty-state');
const clearBtn    = document.getElementById('btn-clear');

const BASE_WIDTH = 560;
const BAR_HEIGHT = 68;
const MAX_HEIGHT = 560;

// How far above true vertical-center the bar should sit when first shown.
// Positive = higher up the screen.
const VERTICAL_OFFSET = 120;

let currentQuery  = '';
let debounceTimer = null;
let anchorLeft    = null; // fixed X so the window never drifts horizontally
let anchorTop     = null; // fixed top edge so growth is downward-only

async function init() {
  // Theme is automatic (prefers-color-scheme), not user-configurable — see
  // css/acdn.css. getConfig() is not currently consumed for any display
  // value; kept as a no-op call so a future setting can be read without
  // wiring up a new IPC round-trip.
  await getConfig().catch(() => {});

  searchInput.addEventListener('input', onSearchInput);
  searchInput.addEventListener('keydown', onSearchKeydown);
  clearBtn?.addEventListener('click', clearSearch);

  // Rust emits this (see main.rs show_query_window) every time the hotkey
  // summons the window. There is no pre-filled/selection-lookup path —
  // this always means "clear the box, focus the input, wait for typing".
  onQuery(() => {
    searchInput.value = '';
    clearBtn && (clearBtn.style.display = 'none');
    searchInput.focus();
    currentQuery = '';
    // Rust just repositioned the window at its above-center anchor for
    // this show — drop our cached anchor so the next resizeToFit() call
    // re-reads the real position instead of reusing a stale one from a
    // previous show (e.g. after the user manually moved/the OS clamped it).
    anchorLeft = null;
    anchorTop = null;
    showEmpty();
  });

  // Spotlight-style: losing focus means the user clicked elsewhere or
  // switched apps — hide immediately rather than staying on screen.
  window.addEventListener('blur', () => {
    hideMainWindow().catch(() => {});
  });

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      if (searchInput.value) {
        clearSearch();
      } else {
        hideMainWindow().catch(() => {});
      }
    }
  });
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
  if (e.key === 'Escape' && searchInput.value) {
    clearSearch();
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
  resultsView.style.display = 'none';
  emptyState.style.display = 'flex';
  resizeToFit();
}

function showResultsView() {
  emptyState.style.display = 'none';
  resultsView.style.display = 'block';
}

// Spotlight-style: the window grows downward to fit results, up to a cap.
// setSize() alone anchors growth at the window's current top-left corner,
// so as long as that corner is fixed the window grows straight down with
// no horizontal drift and no need to re-center (re-centering on every
// keystroke caused visible jumping and a one-frame mismatch between the
// resized window rect and the rounded #app content — the "square corner"
// glitch). The anchor is computed once, the first time the window is
// shown, then reused for every resize until the app restarts.
async function establishAnchor() {
  if (anchorLeft !== null) return;
  const win = getCurrentWindow();
  const [monitor, size, position] = await Promise.all([
    win.currentMonitor(),
    win.outerSize(),
    win.outerPosition(),
  ]);
  if (monitor) {
    const scale = await win.scaleFactor();
    const screenWidth  = monitor.size.width  / scale;
    const screenHeight = monitor.size.height / scale;
    anchorLeft = Math.round((screenWidth - BASE_WIDTH) / 2);
    anchorTop  = Math.round((screenHeight - BAR_HEIGHT) / 2) - VERTICAL_OFFSET;
    anchorTop  = Math.max(anchorTop, 24); // never go off the top edge
  } else {
    // Fallback: keep whatever position the window already has.
    const scale = await win.scaleFactor();
    anchorLeft = Math.round(position.x / scale);
    anchorTop  = Math.round(position.y / scale) - VERTICAL_OFFSET;
  }
  await win.setPosition(new LogicalPosition(anchorLeft, anchorTop));
}

function resizeToFit() {
  requestAnimationFrame(async () => {
    const contentHeight = document.getElementById('app').scrollHeight;
    const height = Math.min(Math.max(contentHeight, BAR_HEIGHT), MAX_HEIGHT);
    const win = getCurrentWindow();
    try {
      await establishAnchor();
      await win.setSize(new LogicalSize(BASE_WIDTH, height));
      // Re-assert position after resize: on Windows, growing a window's
      // size can nudge its top-left if the OS clamps it to stay on-screen.
      await win.setPosition(new LogicalPosition(anchorLeft, anchorTop));
    } catch {
      // best-effort; ignore
    }
  });
}

function renderLoadingState() {
  resultsView.innerHTML = '';
  resultsView.appendChild(renderLoading('Looking up…'));
  resizeToFit();
}

function renderResults(results, query) {
  resultsView.innerHTML = '';

  if (!results || results.length === 0) {
    const noResult = document.createElement('section');
    noResult.className = 'result';
    noResult.innerHTML = `
      <p class="provider-header"><span class="provider-name">No results</span></p>
      <p class="dict-content no-result-text">
        No provider returned a result for <em>${escapeHtml(query)}</em>.
      </p>`;
    resultsView.appendChild(noResult);
    resizeToFit();
    return;
  }

  for (const result of results) {
    resultsView.appendChild(renderResult(result));
  }
  resizeToFit();
}

function renderError(err) {
  resultsView.innerHTML = '';
  const sec = document.createElement('section');
  sec.className = 'result';
  sec.innerHTML = `
    <p class="provider-header"><span class="provider-name error-text">Error</span></p>
    <p class="dict-content no-result-text">${escapeHtml(String(err?.message ?? err))}</p>`;
  resultsView.appendChild(sec);
  resizeToFit();
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

init().catch(err => console.error('[Wispet] init error:', err));
