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

import { lookup, getConfig, hidePopup, showMainWindow, emitOpenQuery } from './ipc.js';
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

  window.addEventListener('blur', () => {
    // Delay lets a click on a button inside the popup register before it hides
    setTimeout(dismiss, 200);
  });

  // Rust (main.rs) calls this via win.eval() to show the popup with a query
  window.onWispetQuery = (query) => {
    handleQuery(query);
  };

  // Query may already be set if this script loads after Rust's eval() call
  if (window.__wispet_query__) {
    handleQuery(window.__wispet_query__);
  }
}

// ── Query handler ─────────────────────────────────────────────────────────────

async function handleQuery(query) {
  if (!query?.trim()) return;

  currentQuery = query.trim();

  if (queryWord) queryWord.textContent = currentQuery;

  if (resultsEl) {
    resultsEl.innerHTML = '';
    resultsEl.appendChild(renderLoading(''));
  }

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
    // eval() isn't available on the JS-side window handle, so broadcast
    // an event instead — see onOpenQuery() in main.js.
    await emitOpenQuery(currentQuery);
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
