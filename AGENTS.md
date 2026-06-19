# AGENTS.md

## Project

- Single-crate Rust 2024 `eframe`/`egui` app named `dreamstack`; native entrypoint and wasm start are both in `src/main.rs`.
- Game data in `data/employers.json` and `data/servers.json` is embedded by `src/game.rs` with `include_str!`; rebuild after changing either file.
- Native autosaves write `autosave.json` via `autosave.json.tmp` in the repo root; wasm autosaves use `localStorage` key `autosave.json`.
- In-game automation scripts are Rhai snippets run through `src/ds.rs`; they must define `fn main(ds)` and use the registered `ds_print(ds, value)` API for captured output.

## Commands

```sh
cargo run                  # debug build + run native app
cargo build --workspace    # CI native debug build shape
cargo build -p dreamstack --target wasm32-unknown-unknown  # CI web build shape
cargo test --workspace     # Rust tests, including inline insta snapshots
cargo test <name>          # focused Rust test by name substring
cargo fmt --check          # CI formatting check
cargo clippy --workspace --all-targets --all-features -- -D warnings
python -m pytest tests     # script tests
cargo test serializes_expected_save_format  # focused inline snapshot test example
cargo insta review         # review newly generated/changed snapshots
cargo insta accept         # accept pending snapshot changes
```

## Testing

- Rust toolchain is pinned to `1.96.0` in `rust-toolchain.toml`; let `rustup` install/use it.
- CI runs `cargo fmt --check`, then clippy with `-D warnings`, then `cargo test --workspace`, plus Python script tests on Linux, Windows, and macOS.
- Rust tests are inline in `src/*.rs` and use `test-case` plus inline `insta` snapshots.
- Run `cargo test` after snapshot edits before `cargo insta review` or `cargo insta accept`.
- Script tests import `scripts/next-dev-tag` directly with Python importlib; keep it executable and side-effect free on import.

## Web

- Web builds require `rustup target add wasm32-unknown-unknown`; browser packaging also needs `wasm-bindgen-cli` matching the locked `wasm-bindgen` crate version.
- Local web run: build the wasm target, run `wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/debug/dreamstack.wasm`, then serve `web/`; `web/index.html` expects a `dreamstack_canvas` element.
- Dev-release CI publishes the web build under GitHub Pages `dev/`; regular release CI does not publish web artifacts.

## Releases

- Package version source of truth is `Cargo.toml`; regular release tags must match `vX.Y.Z` and produce Linux, Windows, macOS, and Linux debug-info artifacts.
- Dev-release tags are `YYwWW[build-id]` or `vX.Y.Z-YYwWW[build-id]`; `scripts/next-dev-tag` prints/creates/pushes the next lowercase suffix tag and rewrites the first `## Unreleased` in `CHANGELOG.md`.
- Release notes come from the first `##` section in `CHANGELOG.md`.
- `scripts/next-dev-tag create` refuses a dirty worktree unless `--force` is passed; when it updates the changelog it commits exactly `chore: update changelog` before tagging.

## Gotchas

- Debug logging reads `RUST_LOG`; release logging uses the fixed filter in `src/log.rs`.
- `.gitignore` ignores dotfiles by default, with `.editorconfig`, `.gitignore`, and `.github` explicitly unignored.
