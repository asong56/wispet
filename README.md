# Wispet

Instant word lookup. Press a hotkey, get a definition, it's gone. Windows
only. Single binary, lives in the system tray.

**Tech stack:** Rust · Tauri v2 · Vanilla HTML/CSS/JS

---

## What it does

- **Lookup** — press `Ctrl+Alt+Space`, the search box appears centered on
  screen, type a word
- The box hides itself when it loses focus or you press `Esc` — it never
  lingers on screen
- **Local MDX dictionary** — offline lookup against a `.mdx` file you supply
  (no dictionary is bundled)
- **DeepL** (optional) — online fallback/translation if you add an API key

There is no in-app settings UI. Everything is configured by hand in
`wispet.toml` — see [Configuration](#configuration) below.

---

## Quick start

### Prerequisites

- [Rust](https://rustup.rs) stable, Windows target
- [Tauri CLI v2](https://tauri.app): `cargo install tauri-cli --version "^2"`

### Run in development

```sh
make dev
```

### Build a release binary

```sh
make release
```

The binary lands at `src-tauri/target/release/wispet.exe`.

---

## Configuration

On first launch, Wispet writes a default config to:

```
%APPDATA%\wispet\wispet.toml
```

Edit that file directly, then restart Wispet — config is read once at
startup, there is no live reload and no settings panel to do it for you.
`config.toml` at the repository root is the template that first-launch
copies from; edit your installed `wispet.toml`, not that template, for a
real setup.

### Pointing it at your dictionary

```toml
[[providers.list]]
type     = "mdx"
enabled  = true
priority = 1
label    = "Vocabulary.com"
path     = "C:\\Dictionaries\\vocabulary.com.mdx"
```

### Adding DeepL (optional)

```toml
[[providers.list]]
type        = "deepl"
enabled     = true
priority    = 2
label       = "DeepL"
api_key     = "your-key-here"   # free tier keys end in ":fx"
source_lang = "auto"
target_lang = "ZH"
```

---

## Default shortcuts

| Action | Default | Configured as |
|---|---|---|
| Open search box | `Ctrl+Alt+Space` | `general.hotkey_main` |
| Clear input / dismiss | `Esc` | fixed |

The hotkey is editable in `wispet.toml`; there is no in-app rebind UI.

If the hotkey does nothing after launch, it likely lost registration to
another app (input-method switchers and launcher tools are common
culprits) — check the log output for a "Shortcut registration failed"
line, then pick a different combination in `wispet.toml`.

---

## Architecture

```
src-tauri/src/
├── main.rs          Tauri builder, window lifecycle, hotkey → query wiring
├── config.rs        wispet.toml read/write, defaults
├── commands.rs      Tauri IPC commands (lookup, get_config, hide)
├── tray.rs          System tray icon and menu
└── provider/
    ├── mod.rs       Provider trait, ProviderResult, dispatch()
    ├── mdx.rs       MDX/MDD local dictionary parser
    └── deepl.rs     DeepL API v2

src/
├── index.html       The one window: search bar + results
├── css/
│   ├── acdn.css     Design tokens + base (system fonts, OKLCH colors, dark mode)
│   └── app.css      Wispet component styles
└── js/
    ├── ipc.js       Tauri invoke/event wrappers
    ├── render.js    Provider result → sanitized DOM
    └── main.js      Window lifecycle, search, resize-to-fit
```

There is exactly one window (`main`). It is summoned at the cursor position,
grows vertically to fit its results up to a cap, and hides on blur or `Esc`.

---

## License

MIT
