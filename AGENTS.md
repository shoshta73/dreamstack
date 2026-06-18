# AGENTS.md

## Project

- Single-crate Rust 2024 `eframe`/`egui` app named `dreamstack`; native entrypoint and wasm start are both in `src/main.rs`.
- Game data in `data/employers.json` and `data/servers.json` is embedded with `include_str!`; rebuild after changing either file.
- Native autosaves write `autosave.json` via `autosave.json.tmp` in the repo root; wasm autosaves use `localStorage` key `autosave.json`.

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
cargo insta review         # review newly generated/changed snapshots
cargo insta accept         # accept pending snapshot changes
```

## Testing

- Rust toolchain is pinned to `1.96.0` in `rust-toolchain.toml`; let `rustup` install/use it.
- CI runs `cargo fmt --check`, then clippy with `-D warnings`, then `cargo test --workspace`, plus Python script tests on Linux, Windows, and macOS.
- Rust tests are inline in `src/*.rs` and use `test-case` plus inline `insta` snapshots.
- Run `cargo test` after snapshot edits before `cargo insta review` or `cargo insta accept`.

## Web

- Web builds require `rustup target add wasm32-unknown-unknown` and `wasm-bindgen-cli` matching the locked `wasm-bindgen` crate version.
- Local web run: build the wasm target, run `wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/debug/dreamstack.wasm`, then serve `web/`.
- Dev-release CI publishes the web build under GitHub Pages `dev/`; regular release CI does not publish web artifacts.

## Releases

- Package version source of truth is `Cargo.toml`; release tags must match `vX.Y.Z`.
- Dev-release tags are `YYwWW[build-id]` or `vX.Y.Z-YYwWW[build-id]`; `scripts/next-dev-tag` can print/create/push the next tag and rewrites the first `## Unreleased` in `CHANGELOG.md`.
- Release notes come from the first `##` section in `CHANGELOG.md`.

## Gotchas

- Debug logging reads `RUST_LOG`; release logging uses the fixed filter in `src/log.rs`.
- `.gitignore` ignores dotfiles with `**/.*`; only `.editorconfig` and `.github` are unignored, and the intended `.gitignore` unignore is misspelled as `!.gitingore`.
