.PHONY: dev build release check fmt lint clean

# ── Development ───────────────────────────────────────────────────────────────

dev:
	cd src-tauri && cargo tauri dev

# ── Build ─────────────────────────────────────────────────────────────────────

build:
	cd src-tauri && cargo tauri build

# tauri build already builds in release mode; there is no --release flag.
release:
	cd src-tauri && cargo tauri build

# ── Quality ───────────────────────────────────────────────────────────────────

check:
	cd src-tauri && cargo check

fmt:
	cd src-tauri && cargo fmt

lint:
	cd src-tauri && cargo clippy -- -D warnings

# ── Clean ─────────────────────────────────────────────────────────────────────

clean:
	cd src-tauri && cargo clean
