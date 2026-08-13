/**
 * settings.js — Settings panel logic.
 * Manages: general (theme, hotkeys), provider list (enable/disable/reorder/API key).
 * All writes go through saveConfig() IPC — no partial saves.
 */

import { saveConfig, getConfigPath } from './ipc.js';

let _config = null;
let _onSaved = null;

// ── Public init ───────────────────────────────────────────────────────────────

/**
 * @param {Config} config
 * @param {(newConfig: Config) => void} onSaved
 */
export async function initSettings(config, onSaved) {
  _config = structuredClone(config);
  _onSaved = onSaved;

  renderSettingsNav();
  renderGeneralPane();
  renderProvidersPane();
  renderAboutPane();

  try {
    const path = await getConfigPath();
    const el = document.getElementById('config-path');
    if (el) el.textContent = path;
  } catch {}

  document.getElementById('settings-save-btn')?.addEventListener('click', onSave);
}

// ── Nav ───────────────────────────────────────────────────────────────────────

const PANES = ['general', 'providers', 'about'];

function renderSettingsNav() {
  const nav = document.querySelector('.settings-nav');
  if (!nav) return;

  const labels = { general: 'General', providers: 'Providers', about: 'About' };
  const icons  = {
    general:   `<svg viewBox="0 0 256 256" fill="currentColor"><path d="M128,82a46,46,0,1,0,46,46A46.06,46.06,0,0,0,128,82Zm0,80a34,34,0,1,1,34-34A34,34,0,0,1,128,162ZM214,130.84c.06-1.89.06-3.79,0-5.68L229.33,106a6,6,0,0,0,1.11-5.29A105.34,105.34,0,0,0,219.76,74.9a6,6,0,0,0-4.53-3l-24.45-2.71q-1.93-2.07-4-4l-2.72-24.46a6,6,0,0,0-3-4.53,105.65,105.65,0,0,0-25.77-10.66A6,6,0,0,0,150,26.68l-19.2,15.37c-1.89-.06-3.79-.06-5.68,0L106,26.67a6,6,0,0,0-5.29-1.11A105.34,105.34,0,0,0,74.9,36.24a6,6,0,0,0-3,4.53L69.23,65.22q-2.07,1.94-4,4L40.76,72a6,6,0,0,0-4.53,3,105.65,105.65,0,0,0-10.66,25.77A6,6,0,0,0,26.68,106l15.37,19.2c-.06,1.89-.06,3.79,0,5.68L26.67,150.05a6,6,0,0,0-1.11,5.29A105.34,105.34,0,0,0,36.24,181.1a6,6,0,0,0,4.53,3l24.45,2.71q1.94,2.07,4,4L72,215.24a6,6,0,0,0,3,4.53,105.65,105.65,0,0,0,25.77,10.66,6,6,0,0,0,5.29-1.11L125.16,214c1.89.06,3.79.06,5.68,0l19.21,15.38a6,6,0,0,0,5.29,1.11,105.34,105.34,0,0,0,25.76-10.68,6,6,0,0,0,3-4.53l2.71-24.45q2.07-1.93,4-4l24.46-2.72a6,6,0,0,0,4.53-3,105.49,105.49,0,0,0,10.66-25.77,6,6,0,0,0-1.11-5.29Z"/></svg>`,
    providers: `<svg viewBox="0 0 256 256" fill="currentColor"><path d="M128,26C75.29,26,34,49.72,34,80v96c0,30.28,41.29,54,94,54s94-23.72,94-54V80C222,49.72,180.71,26,128,26Zm0,12c44.45,0,82,19.23,82,42s-37.55,42-82,42S46,102.77,46,80,83.55,38,128,38Zm82,138c0,22.77-37.55,42-82,42s-82-19.23-82-42V154.79C62,171.16,92.37,182,128,182s66-10.84,82-27.21Zm0-48c0,22.77-37.55,42-82,42s-82-19.23-82-42V106.79C62,123.16,92.37,134,128,134s66-10.84,82-27.21Z"/></svg>`,
    about:     `<svg viewBox="0 0 256 256" fill="currentColor"><path d="M142,176a6,6,0,0,1-6,6,14,14,0,0,1-14-14V128a2,2,0,0,0-2-2,6,6,0,0,1,0-12,14,14,0,0,1,14,14v40a2,2,0,0,0,2,2A6,6,0,0,1,142,176ZM124,94a10,10,0,1,0-10-10A10,10,0,0,0,124,94Zm106,34A102,102,0,1,1,128,26,102.12,102.12,0,0,1,230,128Zm-12,0a90,90,0,1,0-90,90A90.1,90.1,0,0,0,218,128Z"/></svg>`,
  };

  nav.innerHTML = '';
  PANES.forEach((id, i) => {
    const btn = document.createElement('button');
    btn.dataset.pane = id;
    btn.innerHTML = `${icons[id]}<span>${labels[id]}</span>`;
    if (i === 0) btn.classList.add('active');
    btn.addEventListener('click', () => switchPane(id));
    nav.appendChild(btn);
  });
}

function switchPane(id) {
  document.querySelectorAll('.settings-nav button').forEach(b => {
    b.classList.toggle('active', b.dataset.pane === id);
  });
  document.querySelectorAll('.settings-pane').forEach(p => {
    p.style.display = p.dataset.pane === id ? 'block' : 'none';
  });
}

// ── General pane ──────────────────────────────────────────────────────────────

function renderGeneralPane() {
  const pane = document.querySelector('.settings-pane[data-pane="general"]');
  if (!pane) return;

  const g = _config.general;

  pane.innerHTML = `
    <div class="settings-section">
      <h3>Shortcuts</h3>

      <div class="setting-row">
        <div class="setting-label">
          Open Wispet
          <small>Show / hide the main window</small>
        </div>
        <div class="hotkey-input" data-hotkey="hotkey_main" tabindex="0">${g.hotkey_main}</div>
      </div>

      <div class="setting-row">
        <div class="setting-label">
          Translate selection
          <small>Look up current clipboard text</small>
        </div>
        <div class="hotkey-input" data-hotkey="hotkey" tabindex="0">${g.hotkey}</div>
      </div>
    </div>

    <div class="settings-section">
      <h3>Appearance</h3>

      <div class="setting-row">
        <div class="setting-label">Theme</div>
        <div class="theme-switcher">
          <button data-theme-val="light" class="${g.theme === 'light' ? 'active' : ''}">Light</button>
          <button data-theme-val="auto"  class="${g.theme === 'auto'  ? 'active' : ''}">Auto</button>
          <button data-theme-val="dark"  class="${g.theme === 'dark'  ? 'active' : ''}">Dark</button>
        </div>
      </div>
    </div>

    <div class="settings-section">
      <h3>Popup</h3>

      <div class="setting-row">
        <div class="setting-label">
          Auto-dismiss
          <small>Seconds before popup closes (0 = never)</small>
        </div>
        <input
          id="popup-dismiss-input"
          type="number"
          min="0" max="30" step="1"
          value="${Math.round(g.popup_dismiss_ms / 1000)}"
          style="width:60px;font-family:var(--font-mono);font-size:var(--text-sm);
                 padding:5px 8px;border:1px solid var(--color-border);
                 border-radius:var(--radius-base);background:var(--color-surface);
                 color:var(--color-text);outline:none;text-align:center;"
        />
      </div>
    </div>
  `;

  pane.querySelectorAll('.hotkey-input').forEach(el => {
    el.addEventListener('click', () => startHotkeyRecording(el));
    el.addEventListener('keydown', e => { e.preventDefault(); });
  });

  pane.querySelectorAll('.theme-switcher button').forEach(btn => {
    btn.addEventListener('click', () => {
      pane.querySelectorAll('.theme-switcher button').forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      _config.general.theme = btn.dataset.themeVal;
      // Live preview — not saved until onSave()
      applyThemePreview(_config.general.theme);
    });
  });

  pane.querySelector('#popup-dismiss-input')?.addEventListener('change', e => {
    const secs = parseInt(e.target.value) || 0;
    _config.general.popup_dismiss_ms = secs * 1000;
  });
}

function startHotkeyRecording(el) {
  el.classList.add('recording');
  el.textContent = 'Press shortcut…';

  const listener = (e) => {
    e.preventDefault();
    e.stopPropagation();

    const parts = [];
    if (e.ctrlKey)  parts.push('Ctrl');
    if (e.altKey)   parts.push('Alt');
    if (e.shiftKey) parts.push('Shift');
    if (e.metaKey)  parts.push('Super');

    const key = e.key;
    // Ignore lone modifier keys
    if (['Control','Alt','Shift','Meta'].includes(key)) return;

    if (key === 'Escape') {
      cancelRecording(el, listener);
      return;
    }

    const keyName = key.length === 1 ? key.toUpperCase() : key;
    parts.push(keyName);

    const shortcut = parts.join('+');
    el.textContent = shortcut;
    el.classList.remove('recording');

    const field = el.dataset.hotkey; // 'hotkey_main' | 'hotkey'
    _config.general[field] = shortcut;

    document.removeEventListener('keydown', listener, true);
  };

  document.addEventListener('keydown', listener, true);

  const cancel = () => {
    cancelRecording(el, listener);
    document.removeEventListener('click', cancel);
  };
  setTimeout(() => document.addEventListener('click', cancel), 100);
}

function cancelRecording(el, listener) {
  el.classList.remove('recording');
  el.textContent = el.dataset.hotkey === 'hotkey_main'
    ? _config.general.hotkey_main
    : _config.general.hotkey;
  document.removeEventListener('keydown', listener, true);
}

function applyThemePreview(theme) {
  if (theme === 'auto') {
    document.body.removeAttribute('data-theme');
  } else {
    document.body.setAttribute('data-theme', theme);
  }
}

// ── Providers pane ────────────────────────────────────────────────────────────

function renderProvidersPane() {
  const pane = document.querySelector('.settings-pane[data-pane="providers"]');
  if (!pane) return;

  pane.innerHTML = `
    <div class="settings-section">
      <h3>Active providers</h3>
      <p style="font-size:var(--text-xs);color:var(--color-text-muted);margin:0 0 12px">
        Drag to reorder. Results appear in this order.
      </p>
      <div id="provider-list"></div>
      <button class="add-mdx" id="btn-add-mdx">
        <svg viewBox="0 0 256 256" fill="currentColor"><path d="M222,128a6,6,0,0,1-6,6H134v82a6,6,0,0,1-12,0V134H40a6,6,0,0,1,0-12h82V40a6,6,0,0,1,12,0v82h82A6,6,0,0,1,222,128Z"/></svg>
        Add local dictionary (.mdx)
      </button>
    </div>
  `;

  renderProviderList();

  document.getElementById('btn-add-mdx')?.addEventListener('click', addMdxDict);
}

const KIND_ICON_SMALL = {
  mdx: `<svg viewBox="0 0 256 256" fill="currentColor"><path d="M128,26C75.29,26,34,49.72,34,80v96c0,30.28,41.29,54,94,54s94-23.72,94-54V80C222,49.72,180.71,26,128,26Zm0,12c44.45,0,82,19.23,82,42s-37.55,42-82,42S46,102.77,46,80,83.55,38,128,38Zm82,138c0,22.77-37.55,42-82,42s-82-19.23-82-42V154.79C62,171.16,92.37,182,128,182s66-10.84,82-27.21Zm0-48c0,22.77-37.55,42-82,42s-82-19.23-82-42V106.79C62,123.16,92.37,134,128,134s66-10.84,82-27.21Z"/></svg>`,
  deepl: `<svg viewBox="0 0 256 256" fill="currentColor"><path d="M128,26A102,102,0,1,0,230,128,102.12,102.12,0,0,0,128,26Zm89.8,96H173.89c-1.54-40.77-18.48-68.23-30.43-82.67A90.19,90.19,0,0,1,217.8,122ZM128,215.83a110,110,0,0,1-15.19-19.45A128.37,128.37,0,0,1,94.13,134h67.74a128.37,128.37,0,0,1-18.68,62.38A110,110,0,0,1,128,215.83ZM94.13,122a128.37,128.37,0,0,1,18.68-62.38A110,110,0,0,1,128,40.17a110,110,0,0,1,15.19,19.45A128.37,128.37,0,0,1,161.87,122Zm18.41-82.67c-12,14.44-28.89,41.9-30.43,82.67H38.2A90.19,90.19,0,0,1,112.54,39.33ZM38.2,134H82.11c1.54,40.77,18.48,68.23,30.43,82.67A90.19,90.19,0,0,1,38.2,134Zm105.26,82.67c11.95-14.44,28.89-41.9,30.43-82.67H217.8A90.19,90.19,0,0,1,143.46,216.67Z"/></svg>`,
  google: `<svg viewBox="0 0 256 256" fill="currentColor"><path d="M128,26A102,102,0,1,0,230,128,102.12,102.12,0,0,0,128,26Zm89.8,96H173.89c-1.54-40.77-18.48-68.23-30.43-82.67A90.19,90.19,0,0,1,217.8,122ZM128,215.83a110,110,0,0,1-15.19-19.45A128.37,128.37,0,0,1,94.13,134h67.74a128.37,128.37,0,0,1-18.68,62.38A110,110,0,0,1,128,215.83ZM94.13,122a128.37,128.37,0,0,1,18.68-62.38A110,110,0,0,1,128,40.17a110,110,0,0,1,15.19,19.45A128.37,128.37,0,0,1,161.87,122Zm18.41-82.67c-12,14.44-28.89,41.9-30.43,82.67H38.2A90.19,90.19,0,0,1,112.54,39.33ZM38.2,134H82.11c1.54,40.77,18.48,68.23,30.43,82.67A90.19,90.19,0,0,1,38.2,134Zm105.26,82.67c11.95-14.44,28.89-41.9,30.43-82.67H217.8A90.19,90.19,0,0,1,143.46,216.67Z"/></svg>`,
  wikipedia: `<svg viewBox="0 0 256 256" fill="currentColor"><path d="M216,42H40A14,14,0,0,0,26,56V200a14,14,0,0,0,14,14H216a14,14,0,0,0,14-14V56A14,14,0,0,0,216,42Zm2,158a2,2,0,0,1-2,2H40a2,2,0,0,1-2-2V56a2,2,0,0,1,2-2H216a2,2,0,0,1,2,2ZM180,96a6,6,0,0,1-6,6H82a6,6,0,0,1,0-12h92A6,6,0,0,1,180,96Zm0,32a6,6,0,0,1-6,6H82a6,6,0,0,1,0-12h92A6,6,0,0,1,180,128Zm0,32a6,6,0,0,1-6,6H82a6,6,0,0,1,0-12h92A6,6,0,0,1,180,160Z"/></svg>`,
  globe: `<svg viewBox="0 0 256 256" fill="currentColor"><path d="M128,22A106,106,0,1,0,234,128,106.12,106.12,0,0,0,128,22Zm88.44,90H175.7a180.53,180.53,0,0,0-16.24-60.75A94.24,94.24,0,0,1,216.44,112ZM128,222c-9.62-7.06-27.24-31.24-32.7-88h65.4C155.24,190.76,137.62,214.94,128,222Zm-32.7-100c5.46-56.76,23.08-80.94,32.7-88s27.24,31.24,32.7,88ZM96.54,51.25A180.53,180.53,0,0,0,80.3,112H39.56A94.24,94.24,0,0,1,96.54,51.25ZM39.56,134H80.3a180.53,180.53,0,0,0,16.24,60.75A94.24,94.24,0,0,1,39.56,134Zm100.2,60.75A180.53,180.53,0,0,0,175.7,134h40.74A94.24,94.24,0,0,1,159.76,194.75Z"/></svg>`,
};

function renderProviderList() {
  const list = document.getElementById('provider-list');
  if (!list) return;

  const sorted = [..._config.providers.list].sort((a, b) => a.priority - b.priority);
  list.innerHTML = '';

  sorted.forEach((entry, idx) => {
    const label = entry.label ?? entry.type;
    const typeLabel = { mdx: 'Local MDX', deepl: 'DeepL', google: 'Google Translate', wikipedia: 'Wikipedia' }[entry.type] ?? entry.type;

    const item = document.createElement('div');
    item.className = 'provider-item';
    item.draggable = true;
    item.dataset.idx = idx;

    item.innerHTML = `
      <div class="provider-drag-handle" aria-label="Drag to reorder">
        <svg viewBox="0 0 256 256" fill="currentColor"><circle cx="96" cy="60" r="12"/><circle cx="160" cy="60" r="12"/><circle cx="96" cy="128" r="12"/><circle cx="160" cy="128" r="12"/><circle cx="96" cy="196" r="12"/><circle cx="160" cy="196" r="12"/></svg>
      </div>
      <div class="provider-icon">${KIND_ICON_SMALL[entry.type] ?? KIND_ICON_SMALL.globe}</div>
      <div class="provider-meta">
        <strong>${escapeHtml(label)}</strong>
        <small>${typeLabel}${entry.path ? ' · ' + truncatePath(entry.path) : ''}</small>
      </div>
      <label class="toggle" aria-label="Enable ${escapeHtml(label)}">
        <input type="checkbox" ${entry.enabled ? 'checked' : ''}/>
        <div class="toggle-track"></div>
        <div class="toggle-thumb"></div>
      </label>
    `;

    item.querySelector('input[type="checkbox"]').addEventListener('change', e => {
      const realIdx = getProviderRealIndex(entry);
      if (realIdx >= 0) _config.providers.list[realIdx].enabled = e.target.checked;
    });

    if (entry.type === 'deepl') {
      const keyRow = document.createElement('div');
      keyRow.className = 'api-key-row';
      keyRow.innerHTML = `
        <label style="font-size:var(--text-xs);color:var(--color-text-muted);display:block;margin-bottom:4px;">DeepL API Key</label>
        <input type="password" placeholder="xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx:fx" value="${escapeHtml(entry.api_key ?? '')}" />
      `;
      keyRow.querySelector('input').addEventListener('input', e => {
        const realIdx = getProviderRealIndex(entry);
        if (realIdx >= 0) _config.providers.list[realIdx].api_key = e.target.value;
      });
      list.appendChild(item);
      list.appendChild(keyRow);
    } else {
      list.appendChild(item);
    }

    setupDragOnItem(item, sorted, list);
  });
}

// Shared across all drag handlers so dragstart/drop can communicate.
let _dragSrc = null;

function setupDragOnItem(item, sorted, list) {
  item.addEventListener('dragstart', e => {
    _dragSrc = item;
    e.dataTransfer.effectAllowed = 'move';
    item.style.opacity = '0.4';
  });

  item.addEventListener('dragend', () => {
    item.style.opacity = '';
    list.querySelectorAll('.provider-item').forEach(i => i.classList.remove('drag-over'));
    _dragSrc = null;
  });

  item.addEventListener('dragover', e => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    if (item !== _dragSrc) item.classList.add('drag-over');
  });

  item.addEventListener('dragleave', () => item.classList.remove('drag-over'));

  item.addEventListener('drop', e => {
    e.preventDefault();
    if (!_dragSrc || _dragSrc === item) return;

    const srcIdx = parseInt(_dragSrc.dataset.idx);
    const dstIdx = parseInt(item.dataset.idx);

    const reordered = [...sorted];
    const [moved] = reordered.splice(srcIdx, 1);
    reordered.splice(dstIdx, 0, moved);

    reordered.forEach((entry, i) => {
      const realIdx = getProviderRealIndex(entry);
      if (realIdx >= 0) _config.providers.list[realIdx].priority = i + 1;
    });

    renderProviderList();
  });
}

function getProviderRealIndex(entry) {
  return _config.providers.list.findIndex(
    e => e.type === entry.type && (e.label ?? e.type) === (entry.label ?? entry.type) && e.path === entry.path
  );
}

async function addMdxDict() {
  try {
    // Tauri v2 exposes plugin-dialog at window.__TAURI__.dialog
    const { open } = window.__TAURI__.dialog;
    const selected = await open({
      filters: [{ name: 'MDict Dictionary', extensions: ['mdx'] }],
      multiple: false,
    });
    if (!selected) return;

    const path = typeof selected === 'string' ? selected : selected[0];
    const filename = path.split(/[\\/]/).pop().replace(/\.mdx$/i, '');

    const maxPriority = _config.providers.list.reduce((m, e) => Math.max(m, e.priority), 0);
    _config.providers.list.push({
      type: 'mdx',
      enabled: true,
      priority: maxPriority + 1,
      label: filename,
      path,
      api_key: null,
      source_lang: null,
      target_lang: null,
      lang: null,
    });

    renderProviderList();
  } catch (err) {
    console.error('[Wispet] add MDX error:', err);
  }
}

// ── About pane ────────────────────────────────────────────────────────────────

function renderAboutPane() {
  const pane = document.querySelector('.settings-pane[data-pane="about"]');
  if (!pane) return;

  pane.innerHTML = `
    <div class="settings-section">
      <h3>Wispet</h3>
      <div class="setting-row">
        <div class="setting-label">Version</div>
        <span style="font-family:var(--font-mono);font-size:var(--text-xs);color:var(--color-text-muted)">0.1.0</span>
      </div>
      <div class="setting-row">
        <div class="setting-label">
          Config file
          <small>Edit manually for advanced options</small>
        </div>
      </div>
      <div class="config-path" id="config-path">Loading…</div>
    </div>
  `;
}

// ── Save ──────────────────────────────────────────────────────────────────────

async function onSave() {
  const btn = document.getElementById('settings-save-btn');
  if (btn) { btn.textContent = 'Saving…'; btn.disabled = true; }

  try {
    await saveConfig(_config);
    if (btn) { btn.textContent = 'Saved'; btn.style.color = 'var(--color-accent)'; }
    setTimeout(() => {
      if (btn) { btn.textContent = 'Save'; btn.disabled = false; btn.style.color = ''; }
    }, 1500);
    _onSaved?.(_config);
  } catch (err) {
    console.error('[Wispet] save config error:', err);
    if (btn) { btn.textContent = 'Error — retry'; btn.disabled = false; }
  }
}

// ── Util ──────────────────────────────────────────────────────────────────────

function escapeHtml(s) {
  return String(s ?? '')
    .replace(/&/g, '&amp;').replace(/</g, '&lt;')
    .replace(/>/g, '&gt;').replace(/"/g, '&quot;');
}

function truncatePath(p) {
  if (p.length <= 32) return p;
  return '…' + p.slice(-30);
}
