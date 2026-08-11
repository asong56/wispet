import { lookup, getConfig, hidePopup, showMainWindow, emitOpenQuery } from './ipc.js';
import { renderResult, renderLoading } from './render.js';

const queryWord   = document.getElementById('popup-query-word');
const resultsEl   = document.getElementById('popup-results');
const closeBtn    = document.getElementById('popup-close-btn');
const openMainBtn = document.getElementById('popup-open-main-btn');

let dismissTimer = null;
let currentQuery = '';
let dismissMs = 4000;

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
    // Delay lets a click on a button inside the popup register before it hides.
    setTimeout(dismiss, 200);
  });

  // Rust (main.rs) calls this via win.eval() to show the popup with a query
  window.onWispetQuery = (query) => {
    handleQuery(query);
  };

  if (window.__wispet_query__) {
    handleQuery(window.__wispet_query__);
  }
}

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

function applyTheme(theme) {
  if (theme === 'auto') {
    document.body.removeAttribute('data-theme');
  } else {
    document.body.setAttribute('data-theme', theme);
  }
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, '&amp;').replace(/</g, '&lt;')
    .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

init().catch(err => console.error('[Wispet popup] init error:', err));
