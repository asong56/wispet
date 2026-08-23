import { lookup, getConfig, onQuery, hideMainWindow } from './ipc.js';
import { renderResult, renderLoading } from './render.js';

// Tauri v2 renamed window.getCurrent() -> getCurrentWindow()
const { getCurrentWindow, LogicalSize } = window.__TAURI__.window;

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
    showEmpty();
    playEntranceAnimation();
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

// Plays #app's entrance animation exactly once per window show. Restarting
// a CSS animation requires removing the class, forcing reflow, then
// re-adding it — otherwise the browser no-ops a class re-add mid-animation.
function playEntranceAnimation() {
  const app = document.getElementById('app');
  app.classList.remove('entering');
  void app.offsetWidth; // force reflow so the next class add restarts the animation
  app.classList.add('entering');
}

// Spotlight-style: the window grows downward to fit results, up to a cap.
// setSize() alone anchors growth at the window's current top-left corner
// (that corner is set once by Rust in show_query_window, above-center —
// see main.rs), so a plain setSize() here grows the window straight down
// with no horizontal drift. Deliberately NOT calling setPosition/center()
// on every resize: each is a separate async IPC round-trip, and doing two
// of them per keystroke was racing WebView2's own compositor, which is
// what caused the visible tearing/clipping through the search bar.
function resizeToFit() {
  requestAnimationFrame(() => {
    const contentHeight = document.getElementById('app').scrollHeight;
    const height = Math.min(Math.max(contentHeight, BAR_HEIGHT), MAX_HEIGHT);
    getCurrentWindow()
      .setSize(new LogicalSize(BASE_WIDTH, height))
      .catch(() => {});
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
