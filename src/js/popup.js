/**
 * popup.js — Selection-translate popup window.
 *
 * This window is hidden at startup and shown by Rust when the user presses
 * the selection hotkey. It receives the query via window.__wispet_query__
 * (set by Rust via win.eval()) and runs a lookup.
 *
 * Dismiss:
 *   - Esc key
 *   - Click outside (blur event)
 *   - Auto-dismiss timer (from config)
 *   - "Open in Wispet" button → shows main window with same query
 */

import { lookup, getConfig, hidePopup, showMainWindow } from './ipc.js';
import { renderResult, renderLoading } from './render.js';

// ── DOM refs ──────────────────────────────────────────────────────────────────

const queryWord   = document.getElementById('popup-query-word');
const resultsEl   = document.getElementById('popup-results');
const closeBtn    = document.getElementById('popup-close-btn');
const openMainBtn = document.getElementById('popup-open-main-btn');

// ── State ─────────────────────────────────────────────────────────────────────

let dismissTimer = null;
let currentQuery = '';
let dismissMs = 4000;

// ── Init ──────────────────────────────────────────────────────────────────────

async function init() {
  try {
    const config = await getConfig();
    dismissMs = config.general.popup_dismiss_ms ?? 4000;
    applyTheme(config.general.theme);
  } catch {}

  closeBtn?.addEventListener('click', dismiss);
  openMainBtn?.addEventListener('click', openInMain);

  document.addEventListener('keydown', e => {
    if (e.key === 'Escape') dismiss();
  });

  // Tauri: hide on blur
  window.addEventListener('blur', () => {
    // Small delay to allow button clicks inside to register
    setTimeout(dismiss, 200);
  });

  // Rust calls this when it wants to show the popup with a query
  window.onWispetQuery = (query) => {
    handleQuery(query);
  };

  // If query was already set before this script loaded
  if (window.__wispet_query__) {
    handleQuery(window.__wispet_query__);
  }
}

// ── Query handler ─────────────────────────────────────────────────────────────

async function handleQuery(query) {
  if (!query?.trim()) return;

  currentQuery = query.trim();

  // Update header
  if (queryWord) queryWord.textContent = currentQuery;

  // Show loading
  if (resultsEl) {
    resultsEl.innerHTML = '';
    resultsEl.appendChild(renderLoading(''));
  }

  // Reset dismiss timer
  resetDismissTimer();

  try {
    const results = await lookup(currentQuery);
    if (!resultsEl) return;
    resultsEl.innerHTML = '';

    if (!results || results.length === 0) {
      resultsEl.innerHTML = `
        <section class="result">
          <div class="dict-content" style="color:var(--color-text-muted);font-size:var(--text-sm);padding:4px 0">
            No result for <em>${escapeHtml(currentQuery)}</em>
          </div>
        </section>`;
      return;
    }

    for (const r of results) {
      resultsEl.appendChild(renderResult(r));
    }

    // Only reset the timer if the popup is still showing the same query.
    // If the user blurred the window while results were loading, dismiss()
    // already cleared dismissTimer; don't restart it here.
  } catch (err) {
    if (resultsEl) {
      resultsEl.innerHTML = `
        <section class="result">
          <div class="dict-content" style="color:var(--color-text-muted);font-size:var(--text-sm);padding:4px 0">
            ${escapeHtml(String(err?.message ?? err))}
          </div>
        </section>`;
    }
  }
}

// ── Dismiss ───────────────────────────────────────────────────────────────────

function dismiss() {
  clearTimeout(dismissTimer);
  hidePopup().catch(() => {});
}

function resetDismissTimer() {
  clearTimeout(dismissTimer);
  if (dismissMs > 0) {
    dismissTimer = setTimeout(dismiss, dismissMs);
  }
}

// ── Open in main window ───────────────────────────────────────────────────────

async function openInMain() {
  clearTimeout(dismissTimer);
  try {
    await showMainWindow();
    // Signal the main window to search for the current query
    // Tauri v2: WebviewWindow.getByLabel is on the WebviewWindow class
    const { WebviewWindow } = window.__TAURI__.webviewWindow;
    const mainWin = await WebviewWindow.getByLabel('main');
    await mainWin?.eval(
      `document.getElementById('search-input').value = ${JSON.stringify(currentQuery)};
       document.getElementById('search-input').dispatchEvent(new Event('input'));`
    );
  } catch {}
  dismiss();
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
    .replace(/&/g, '&amp;').replace(/</g, '&lt;')
    .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

// ── Boot ──────────────────────────────────────────────────────────────────────

init().catch(err => console.error('[Wispet popup] init error:', err));
