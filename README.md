# Wispet

Minimal, fast dictionary and translation tool. Single binary. Lives in your system tray.

**Tech stack:** Rust · Tauri v2 · Vanilla HTML/CSS/JS · ACDN design system

---

## Features

- **Main window** — Raycast-style search: type, get results instantly
- **Selection translate** — select any text, press `Alt+D`, get a floating result
- **Local MDX dictionaries** — fast offline lookup, zero network, supports MDD media
- **Multiple providers** — MDX, DeepL, Google Translate, Wikipedia, in priority order
- **Single binary** — one file, no installer, no runtime dependencies
- **System tray** — runs quietly in the background

---

## Quick start

### Prerequisites

- [Rust](https://rustup.rs) stable
- [Tauri CLI v2](https://tauri.app): `cargo install tauri-cli --version "^2"`
- Platform webview: WebKit2GTK on Linux, built-in on macOS/Windows

### Run in development

```sh
make dev
```

### Build a release binary

```sh
make release
```

The binary lands in `src-tauri/target/release/wispet` (Linux/macOS) or `wispet.exe` (Windows).

---

## Configuration

Config file is created at first launch:

| Platform | Path |
|----------|------|
| macOS    | `~/Library/Application Support/wispet/config.toml` |
| Linux    | `~/.config/wispet/config.toml` |
| Windows  | `%APPDATA%\wispet\config.toml` |

You can also open it from **Settings → About → Config file**.

### Adding a local dictionary

```toml
[[providers.list]]
type     = "mdx"
enabled  = true
priority = 1
label    = "Oxford Advanced"
path     = "/path/to/OALD10.mdx"
```

Or use **Settings → Providers → Add local dictionary** for a file picker.

---

## Providers

| Provider | Type | Notes |
|----------|------|-------|
| Local MDX | `mdx` | Offline. Place `.mdx` and optional `.mdd` in the same directory |
| DeepL | `deepl` | Requires API key (free tier `:fx` supported) |
| Google Translate | `google` | Unofficial public endpoint, no key required |
| Wikipedia | `wikipedia` | Summary endpoint, configurable language |

---

## Default shortcuts

| Action | Default |
|--------|---------|
| Open / hide main window | `Alt+W` |
| Translate selection | `Alt+D` |
| Clear search | `Esc` |
| Open settings | `⌘,` / `Ctrl+,` |

Shortcuts are configurable in **Settings → General**.

---

## Architecture

```
src-tauri/src/
├── main.rs          Tauri builder, shortcuts, tray, clipboard watcher
├── config.rs        config.toml read/write
├── commands.rs      Tauri IPC commands (invoke handlers)
├── clipboard.rs     Cross-platform clipboard change watcher thread
├── tray.rs          System tray icon and menu
└── provider/
    ├── mod.rs       Provider trait, ProviderResult, dispatch()
    ├── mdx.rs       MDX/MDD local dictionary parser
    ├── deepl.rs     DeepL API v2
    ├── google.rs    Google Translate (unofficial)
    └── wikipedia.rs Wikipedia REST API

src/
├── index.html       Main window
├── popup.html       Selection-translate floating popup
├── css/
│   ├── acdn.css     ACDN design tokens + base (compiled from SCSS)
│   └── app.css      Wispet component styles
└── js/
    ├── ipc.js       Tauri invoke/event wrappers
    ├── render.js    Provider result → DOM (shared by main + popup)
    ├── main.js      Main window logic
    ├── settings.js  Settings panel
    └── popup.js     Popup logic
```

---

## License

MIT
